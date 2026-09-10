extern crate std;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use alloc::{collections::BTreeMap, string::String, sync::Arc, vec};

use serde::Deserialize;
use spin::Mutex;
use wie_arm_jit::{Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion};

use super::*;

#[derive(Deserialize)]
struct NodeResult {
    frame: [u32; 24],
    effects: Vec<NodeEffect>,
    exit: u32,
}

#[derive(Deserialize)]
enum NodeEffect {
    Memory(Vec<(u32, Vec<u8>)>),
    Sample([u32; 4]),
    Store([u32; 3]),
    WordRange([u32; 3]),
}

#[derive(Default)]
struct NodeExecutor {
    modules: BTreeMap<u64, Vec<u8>>,
    completion: Option<CompileCompletion>,
    calls: Arc<Mutex<u32>>,
    retired: Arc<Mutex<u32>>,
    helper_stores: Arc<Mutex<u32>>,
    handoffs: Arc<Mutex<Vec<RunFrame>>>,
}

impl CompiledExecutor for NodeExecutor {
    fn submit(&mut self, request: &CompileRequest) -> Admission {
        let artifact = wie_arm_wasm::compile(request).unwrap();
        let size = artifact.bytes.len();
        self.modules.insert(request.request, artifact.bytes);
        self.completion = Some(CompileCompletion {
            session: request.session,
            request: request.request,
            result: Ok(CompiledArtifact {
                encoded_size: size,
                regions: artifact
                    .manifest
                    .into_iter()
                    .enumerate()
                    .map(|(slot, manifest)| CompiledRegion {
                        manifest,
                        handle: CompiledHandle {
                            slot: slot as u32,
                            generation: request.request,
                        },
                    })
                    .collect(),
            }),
        });
        Admission::Accepted
    }

    fn poll(&mut self) -> Option<CompileCompletion> {
        self.completion.take()
    }

    fn execute(
        &mut self,
        handle: CompiledHandle,
        frame: &mut RunFrame,
        access: &mut dyn ExecutionAccess,
    ) -> core::result::Result<CompiledExit, String> {
        *self.calls.lock() += 1;
        let mut pages = Vec::new();
        for base in [0, 0x1000, 0x10000, 0x20000, 0x2ff00, 0x30000, 0xffff_ff00] {
            let mut bytes = Vec::new();
            for offset in (0..256).step_by(4) {
                let AccessResult::Complete(value) = access.load(base + offset, 4) else {
                    break;
                };
                bytes.extend_from_slice(&value.to_le_bytes());
            }
            if !bytes.is_empty() {
                pages.push((base, bytes));
            }
        }
        let mut child = Command::new("node")
            .args(["-e", include_str!("differential.js")])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let payload = serde_json::json!({
            "wasm": self.modules[&handle.generation],
            "pages": pages,
            "slot": handle.slot,
            "frame": bytemuck::cast_slice::<RunFrame, u32>(core::slice::from_ref(frame)),
        });
        child.stdin.take().unwrap().write_all(&serde_json::to_vec(&payload).unwrap()).unwrap();
        let output = child.wait_with_output().unwrap();
        assert!(
            output.status.success(),
            "generated Wasm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result: NodeResult = serde_json::from_slice(&output.stdout).unwrap();
        for effect in result.effects {
            match effect {
                NodeEffect::Memory(pages) => {
                    for (base, bytes) in pages {
                        for (index, word) in bytes.as_chunks::<4>().0.iter().enumerate() {
                            assert!(matches!(
                                access.store(base + index as u32 * 4, 4, u32::from_le_bytes(*word)),
                                AccessResult::Complete(0)
                            ));
                        }
                    }
                }
                NodeEffect::Sample([pc, cpsr, r7, entry_pc]) => access.sample_prepare(pc, cpsr, r7, entry_pc),
                NodeEffect::Store([address, width, value]) => {
                    assert!(matches!(access.store(address, width, value), AccessResult::Complete(0)));
                    *self.helper_stores.lock() += 1;
                }
                NodeEffect::WordRange([address, words, admitted]) => assert_eq!(u32::from(access.word_range(address, words).is_some()), admitted),
            }
        }
        *frame = bytemuck::pod_read_unaligned(bytemuck::cast_slice(&result.frame));
        *self.retired.lock() += frame.executed;
        if result.exit == CompiledExit::InterpretOne as u32 {
            self.handoffs.lock().push(*frame);
        }
        Ok(match result.exit {
            0 => CompiledExit::Dispatch,
            1 => CompiledExit::Sample,
            2 => CompiledExit::Budget,
            3 => CompiledExit::End,
            4 => CompiledExit::InterpretOne,
            6 => CompiledExit::GuestFault,
            other => panic!("invalid generated exit {other}"),
        })
    }

    fn retire(&mut self, handles: &[CompiledHandle]) {
        for handle in handles {
            self.modules.remove(&handle.generation);
        }
    }

    fn shutdown(&mut self) {
        self.modules.clear();
        self.completion = None;
    }
}

struct DifferentialRun {
    engine: Arm32CpuEngine,
    result: Result<EngineRunResult>,
    retired: u32,
    helper_stores: u32,
    handoffs: Vec<RunFrame>,
}

fn run_decoded(code: &[u8], cpsr: u32, setup: impl Fn(&mut Arm32CpuEngine), end: u32, budget: u32, interval: u32) -> DifferentialRun {
    let mut interpreter = Arm32CpuEngine::new();
    let mut compiled = Arm32CpuEngine::new();
    for engine in [&mut interpreter, &mut compiled] {
        engine.mem_map(0x1000, code.len() + 4, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, code).unwrap();
        let frontier = if cpsr & 0x20 != 0 { 0xde00_de00u32 } else { 0xe7f0_00f0 };
        engine.mem_write(0x1000 + code.len() as u32, &frontier.to_le_bytes()).unwrap();
        engine.reg_write(ArmRegister::Cpsr, cpsr);
        engine.reg_write(ArmRegister::PC, 0x1000);
        engine.sampler.remaining = interval;
        engine.set_profiling(true);
        setup(engine);
        engine.mark_entry();
    }
    let executor = NodeExecutor::default();
    let calls = executor.calls.clone();
    let retired = executor.retired.clone();
    let helper_stores = executor.helper_stores.clone();
    let handoffs = executor.handoffs.clone();
    let mut jit = Jit::new(1, Box::new(executor));
    let entry = RegionKey {
        pc: compiled.reg_read(ArmRegister::PC),
        thumb: cpsr & 0x20 != 0,
        cpu_mode: (cpsr & 0x1f) as u8,
    };
    jit.sample(entry, Some((entry, 8)));
    jit.maintain(&compiled.mem);
    compiled.jit = Some(jit);
    let expected = interpreter.run(end, budget, None);
    let result = compiled.run(end, budget, None);
    assert!(
        *calls.lock() > 0 || budget == 0 || end == entry.pc,
        "decoder did not compile {code:x?}, cpsr={cpsr:#x}"
    );
    match (&result, &expected) {
        (Ok(actual), Ok(expected)) => {
            assert_eq!(actual.instructions_executed, expected.instructions_executed, "{code:x?}");
            assert_eq!(
                core::mem::discriminant(&actual.stop_reason),
                core::mem::discriminant(&expected.stop_reason)
            );
        }
        (Err(actual), Err(expected)) => assert_eq!(format!("{actual:?}"), format!("{expected:?}"), "{code:x?}"),
        _ => panic!(
            "run outcomes differ for {code:x?}: compiled={:?}, interpreted={:?}",
            result.as_ref().err(),
            expected.as_ref().err()
        ),
    }
    assert_eq!(compiled.cpu, interpreter.cpu, "whole CPU, code={code:x?}, cpsr={cpsr:#x}");
    assert_eq!(compiled.entry_pc, interpreter.entry_pc, "observed entry, code={code:x?}, cpsr={cpsr:#x}");
    for (page, (actual, expected)) in compiled.mem.pages.iter().zip(interpreter.mem.pages.iter()).enumerate() {
        assert_eq!(
            actual.as_ref().map(|page| page.bytes.as_slice()),
            expected.as_ref().map(|page| page.bytes.as_slice()),
            "memory page {page:#x}, code={code:x?}"
        );
    }
    assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining, "{code:x?}");
    assert_eq!(compiled.sampler.sequence, interpreter.sampler.sequence, "{code:x?}");
    let actual: Vec<_> = compiled
        .take_profile(true)
        .into_iter()
        .map(|sample| (sample.stack, sample.count))
        .collect();
    let expected: Vec<_> = interpreter
        .take_profile(true)
        .into_iter()
        .map(|sample| (sample.stack, sample.count))
        .collect();
    assert_eq!(actual, expected, "samples, code={code:x?}");
    DifferentialRun {
        engine: compiled,
        result,
        retired: *retired.lock(),
        helper_stores: *helper_stores.lock(),
        handoffs: handoffs.lock().clone(),
    }
}

#[test]
fn queued_arm_and_thumb_regions_execute_from_one_wasm_module() {
    let mut interpreter = Arm32CpuEngine::new();
    let mut compiled = Arm32CpuEngine::new();
    let arm: Vec<_> = [0xe1a0400eu32, 0xeb000005, 0xe2800001, 0xe12fff14]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect();
    let thumb: Vec<_> = [0x4674u16, 0xf000, 0xf80d, 0x3002, 0x4720]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect();
    for engine in [&mut interpreter, &mut compiled] {
        engine.mem_map(0x1000, 256, MemoryPermission::ReadWriteExecute);
        // Save LR; call the local callee; add again after returning; bx saved LR.
        engine.mem_write(0x1000, &arm).unwrap();
        engine.mem_write(0x1020, &[0x03, 0x00, 0x80, 0xe2, 0x1e, 0xff, 0x2f, 0xe1]).unwrap();
        engine.mem_write(0x1080, &thumb).unwrap();
        engine.mem_write(0x10a0, &[0x05, 0x30, 0x70, 0x47]).unwrap();
    }
    let entries = [
        RegionKey {
            pc: 0x1000,
            thumb: false,
            cpu_mode: 0x1f,
        },
        RegionKey {
            pc: 0x1080,
            thumb: true,
            cpu_mode: 0x10,
        },
    ];
    let executor = NodeExecutor::default();
    let calls = executor.calls.clone();
    let retired = executor.retired.clone();
    let mut jit = Jit::new(1, Box::new(executor));
    for entry in entries {
        jit.sample(entry, Some((entry, 8)));
    }
    jit.maintain(&compiled.mem);
    jit.poll(&compiled.mem);
    let first = jit.lookup(entries[0], &compiled.mem).unwrap();
    let second = jit.lookup(entries[1], &compiled.mem).expect("both queued regions must install together");
    assert_eq!(first.generation, second.generation);
    assert_ne!(first.slot, second.slot);
    for (entry, continuation, callee, handle) in [(entries[0], 0x1008, 0x1020, first), (entries[1], 0x1086, 0x10a0, second)] {
        assert_eq!(jit.lookup(RegionKey { pc: continuation, ..entry }, &compiled.mem), Some(handle));
        assert!(jit.lookup(RegionKey { pc: callee, ..entry }, &compiled.mem).is_none());
    }
    compiled.jit = Some(jit);
    for entry in entries {
        for engine in [&mut interpreter, &mut compiled] {
            engine.reg_write(ArmRegister::Cpsr, u32::from(entry.cpu_mode) | if entry.thumb { 0x20 } else { 0 });
            engine.reg_write(ArmRegister::PC, entry.pc);
            engine.reg_write(ArmRegister::LR, 0x2000 | u32::from(entry.thumb));
            engine.reg_write(ArmRegister::R0, 41);
        }
        let expected = interpreter.run(0x2000, 10, None).unwrap();
        let actual = compiled.run(0x2000, 10, None).unwrap();
        assert_eq!(actual.instructions_executed, 6);
        assert_eq!(actual.instructions_executed, expected.instructions_executed);
        assert!(matches!(actual.stop_reason, EngineStopReason::End));
        assert!(matches!(expected.stop_reason, EngineStopReason::End));
        assert_eq!(compiled.cpu, interpreter.cpu);
        assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
    }
    // Each caller dispatches its callee to the interpreter and resumes at its continuation.
    assert_eq!(*calls.lock(), 4);
    assert_eq!(*retired.lock(), 8);
}

#[test]
fn decoded_mla_and_long_multiply_cover_aliases_conditions_and_status_masks() {
    let mut opcodes = vec![0xe023_2190u32, 0xe030_2190, 0xe031_2190, 0xe032_2190, 0x0032_0190, 0x1030_0190];
    for signed in [false, true] {
        for accumulate in [false, true] {
            for set_flags in [false, true] {
                for pair in [0x0003_2000, 0x0001_0000] {
                    opcodes.push(0xe080_0190 | pair | (u32::from(signed) << 22) | (u32::from(accumulate) << 21) | (u32::from(set_flags) << 20));
                }
            }
        }
    }
    opcodes.extend([0x0093_2190, 0x10f1_0190]);
    for opcode in opcodes {
        for (values, flags) in [
            ([0, 1, 0, 0], 0x3800_0000),
            ([u32::MAX, 2, 2, 0], 0x9800_0000),
            ([0x8000_0000, 2, u32::MAX, 0x7fff_ffff], 0xa800_0000),
            ([u32::MAX, u32::MAX, u32::MAX, u32::MAX], 0x4800_0000),
        ] {
            for mode in [0x10, 0x1f] {
                let run = run_decoded(
                    &opcode.to_le_bytes(),
                    flags | mode,
                    |engine| {
                        for (register, value) in values.into_iter().enumerate() {
                            engine.cpu.reg_set(Mode::User, register as u8, value);
                        }
                    },
                    0x1004,
                    1,
                    1,
                );
                assert_eq!(run.retired, 1, "opcode={opcode:#x}");
                assert_eq!(run.result.unwrap().instructions_executed, 1);
                assert!(run.handoffs.is_empty());
                assert_eq!(run.engine.reg_read(ArmRegister::Cpsr) & 0x3fff_ffff, (flags | mode) & 0x3fff_ffff);
            }
        }
    }
}

#[test]
fn decoded_clz_and_user_status_access_retires_in_both_modes() {
    for opcode in [
        0xe16f_0f10u32,
        0xe16f_2f10,
        0x016f_0f10,
        0xe10f_0000,
        0xe10f_e000,
        0x010f_0000,
        0xe128_f000,
        0xe12c_f000,
        0xe328_f480,
        0x0128_f000,
    ] {
        for value in [0u32, 1, 0x8000_0000, 0xffff_ffff] {
            for mode in [0x10, 0x1f] {
                let run = run_decoded(
                    &opcode.to_le_bytes(),
                    0xa800_00c0 | mode,
                    |engine| {
                        engine.reg_write(ArmRegister::R0, value);
                    },
                    0x1004,
                    1,
                    1,
                );
                assert_eq!(run.retired, 1, "opcode={opcode:#x}");
                assert_eq!(run.result.unwrap().instructions_executed, 1);
                assert!(run.handoffs.is_empty());
                assert_eq!(run.engine.reg_read(ArmRegister::Cpsr) & 0x0fff_ffff, 0x0800_00c0 | mode);
            }
        }
    }
    for opcode in [0xe121_f000u32, 0xe129_f000, 0xe321_f013] {
        let run = run_decoded(
            &opcode.to_le_bytes(),
            0x3800_0010,
            |engine| {
                engine.reg_write(ArmRegister::R0, 0xf000_0013);
            },
            0x1004,
            1,
            1,
        );
        assert_eq!(run.retired, 1);
        assert_eq!(run.result.unwrap().instructions_executed, 1);
        assert_eq!(run.engine.reg_read(ArmRegister::Cpsr) & 0x0fff_ffff, 0x0800_0010);
    }
}

#[test]
fn decoded_arm_pc_operands_and_branches_preserve_state_and_links() {
    for (opcode, r0, r1, lr) in [
        (0xe1a0_f000u32, 0x20000, 0, 0x30000), // mov pc, r0
        (0xe280_f004, 0x1fffc, 0, 0x30000),    // add pc, r0, #4
        (0xe28f_f000, 0, 0, 0x30000),          // add pc, pc, #0, constant folded
        (0xe08f_f080, 4, 0, 0x30000),          // add pc, pc, r0, lsl #1
        (0xe1a0_208f, 0, 0, 0x30000),          // mov r2, pc, lsl #1
        (0xe08f_2080, 4, 0, 0x30000),          // add r2, pc, r0, lsl #1
        (0xe08f_2000, 4, 0, 0x30000),          // add r2, pc, r0
        (0x01a0_f000, 0x20000, 0, 0x30000),    // condition false
        (0xe12f_ff10, 0x20000, 0, 0x30000),
        (0xe12f_ff10, 0x20003, 0, 0x30000),
        (0xe12f_ff1f, 0, 0, 0x30000),
        (0xe12f_ff30, 0x20000, 0, 0x30000),
        (0xe12f_ff30, 0x20003, 0, 0x30000),
        (0xe12f_ff3e, 0, 0, 0x20003), // target aliases LR
        (0x012f_ff3e, 0, 0, 0x20003),
        (0xeb00_03fe, 0, 0, 0x30000),
        (0xfa00_03fe, 0, 0, 0x30000),
        (0xfb00_03fe, 0, 0, 0x30000),
        (0xeaff_fffe, 0, 0, 0x30000),
    ] {
        for mode in [0x10, 0x1f] {
            let run = run_decoded(
                &opcode.to_le_bytes(),
                0x3800_0000 | mode,
                |engine| {
                    engine.reg_write(ArmRegister::R0, r0);
                    engine.reg_write(ArmRegister::R1, r1);
                    engine.reg_write(ArmRegister::LR, lr);
                },
                0x1004,
                1,
                1,
            );
            assert_eq!(run.retired, 1, "opcode={opcode:#x}");
            assert_eq!(run.result.unwrap().instructions_executed, 1);
            assert!(run.handoffs.is_empty());
        }
    }
}

#[test]
fn decoded_thumb_pc_branches_and_call_pairs_retire_as_cpu_instructions() {
    for (code, r0, lr, retired) in [
        (vec![0x4687u16], 0x20001, 0x30000, 1), // mov pc, r0
        (vec![0x4487], 0x1effc, 0x30000, 1),    // add pc, r0
        (vec![0x4678], 0, 0x30000, 1),          // mov r0, pc
        (vec![0x4478], 4, 0x30000, 1),          // add r0, pc
        (vec![0x4778], 0, 0x30000, 1),          // bx pc
        (vec![0x4700], 0x20000, 0x30000, 1),
        (vec![0x4700], 0x20003, 0x30000, 1),
        (vec![0x4780], 0x20000, 0x30000, 1),
        (vec![0x4780], 0x20003, 0x30000, 1),
        (vec![0x47f0], 0, 0x20003, 1),         // blx lr
        (vec![0xf000, 0xf800], 0, 0x30000, 1), // bl pair
        (vec![0xf7ff, 0xfffe], 0, 0x30000, 1), // backward bl pair
        (vec![0xf000, 0xe800], 0, 0x30000, 1), // blx pair
        (vec![0xf000, 0xe802], 0, 0x30000, 1),
        (vec![0x46c0, 0xf000, 0xe800], 0, 0x30000, 2), // pair starts at a halfword-aligned PC
    ] {
        let code = code.into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>();
        for mode in [0x10, 0x1f] {
            let mut run = run_decoded(
                &code,
                0x3800_0020 | mode,
                |engine| {
                    engine.reg_write(ArmRegister::R0, r0);
                    engine.reg_write(ArmRegister::LR, lr);
                },
                0x30000,
                retired,
                retired,
            );
            assert_eq!(run.retired, retired, "{code:x?}");
            assert_eq!(run.result.unwrap().instructions_executed, retired);
            assert!(run.handoffs.is_empty());
            if code.len() >= 4 {
                let suffix = RegionKey {
                    pc: 0x1000 + code.len() as u32 - 2,
                    thumb: true,
                    cpu_mode: mode as u8,
                };
                assert!(run.engine.jit.as_mut().unwrap().lookup(suffix, &run.engine.mem).is_none());
            }
        }
    }
}

#[test]
fn decoded_thumb_call_at_source_boundary_executes_whole_pair_in_cpu() {
    for suffix in [0xf802u16, 0xe802] {
        let code = [0x46c0, 0xf000, suffix].into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>();
        for budget in 0..=2 {
            for interval in 1..=3 {
                let mut run = run_decoded(
                    &code,
                    0x3800_003f,
                    |engine| {
                        engine.mem_write(0x3ffc, &code).unwrap();
                        engine.reg_write(ArmRegister::PC, 0x3ffc);
                        engine.reg_write(ArmRegister::LR, 0x30000);
                    },
                    0x50000,
                    budget,
                    interval,
                );
                assert_eq!(run.retired, budget.min(1));
                assert_eq!(run.result.unwrap().instructions_executed, budget);
                assert!(run.handoffs.is_empty());
                let expected_pc = match budget {
                    0 => 0x3ffc,
                    1 => 0x3ffe,
                    _ if suffix == 0xf802 => 0x4006,
                    _ => 0x4004,
                };
                assert_eq!(run.engine.reg_read(ArmRegister::PC), expected_pc);
                assert_eq!(run.engine.reg_read(ArmRegister::LR), if budget == 2 { 0x4003 } else { 0x30000 });
                let suffix = RegionKey {
                    pc: 0x4000,
                    thumb: true,
                    cpu_mode: 0x1f,
                };
                assert!(run.engine.jit.as_mut().unwrap().lookup(suffix, &run.engine.mem).is_none());
            }
        }
    }
}

#[test]
fn decoded_multiple_transfers_cover_addressing_masks_aliases_and_pc() {
    for addressing in [0u32, 0x0080_0000, 0x0100_0000, 0x0180_0000] {
        for (list, load, writeback) in [
            (0x0025, false, false),
            (0x0025, false, true),
            (0x0025, true, true),
            (0x8005, false, true),
            (0x8005, true, true),
            (0x0045, true, false),
            (0x00c0, false, true),
        ] {
            for condition in [0xe000_0000, 0] {
                let opcode = condition | 0x0806_0000 | addressing | list | (u32::from(load) << 20) | (u32::from(writeback) << 21);
                let run = run_decoded(
                    &opcode.to_le_bytes(),
                    0x3800_0010,
                    |engine| {
                        engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                        let data: Vec<_> = (0..64u32).flat_map(|index| (0x20003 + index * 4).to_le_bytes()).collect();
                        engine.mem_write(0x20000, &data).unwrap();
                        engine.reg_write(ArmRegister::R0, 0x1122_3344);
                        engine.reg_write(ArmRegister::R2, 0x5566_7788);
                        engine.reg_write(ArmRegister::R5, 0x99aa_bbcc);
                        engine.reg_write(ArmRegister::R6, 0x20040);
                        engine.reg_write(ArmRegister::R7, 0);
                    },
                    0x1004,
                    1,
                    1,
                );
                assert_eq!(run.retired, 1, "opcode={opcode:#x}");
                assert_eq!(run.result.unwrap().instructions_executed, 1);
                assert_eq!(run.helper_stores, 0);
                assert!(run.handoffs.is_empty());
            }
        }
    }
    for opcode in [0xc625u16, 0xce25, 0xce45, 0xc6c0, 0xb525, 0xbc25, 0xbd25] {
        for target in [0x20000u32, 0x20003] {
            let run = run_decoded(
                &opcode.to_le_bytes(),
                0x3800_003f,
                |engine| {
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    let data: Vec<_> = (0..64u32).flat_map(|index| (target + index * 4).to_le_bytes()).collect();
                    engine.mem_write(0x20000, &data).unwrap();
                    engine.reg_write(ArmRegister::R0, 0x1122_3344);
                    engine.reg_write(ArmRegister::R2, 0x5566_7788);
                    engine.reg_write(ArmRegister::R5, 0x99aa_bbcc);
                    engine.reg_write(ArmRegister::R6, 0x20040);
                    engine.reg_write(ArmRegister::R7, 0);
                    engine.reg_write(ArmRegister::SP, 0x20080);
                    engine.reg_write(ArmRegister::LR, 0x30001);
                },
                0x1002,
                1,
                1,
            );
            assert_eq!(run.retired, 1, "opcode={opcode:#x}");
            assert_eq!(run.result.unwrap().instructions_executed, 1);
            assert!(run.handoffs.is_empty());
            assert_eq!(run.helper_stores, 0);
        }
    }
}

#[test]
fn decoded_swaps_doublewords_and_pc_loads_preserve_memory_and_aliases() {
    let mut cases = vec![
        (0xe590_f008u32, 0),
        (0xe530_f008, 0),
        (0xe490_f008, 0),
        (0xe790_f102, 0),
        (0x0590_f008, 0),
        (0xe580_f000, 1), // str pc, [r0]
        (0xe100_1092, 0),
        (0xe100_1091, 0),
        (0xe140_1092, 1),
        (0xe140_1091, 1),
        (0x0100_1092, 0),
    ];
    for addressing in [0x0140_0000u32, 0x0160_0000, 0x0040_0000, 0x0100_0000, 0x0120_0000, 0] {
        for up in [false, true] {
            for load in [false, true] {
                let offset = if addressing & 0x0040_0000 != 0 { 0x108 } else { 2 };
                cases.push((0xe000_40d0 | addressing | (u32::from(up) << 23) | (u32::from(!load) << 5) | offset, 0));
            }
        }
    }
    cases.extend([(0xe1c0_00d0, 0), (0xe1c0_00f0, 0), (0xe180_40f4, 0), (0x01c0_40d0, 0)]);
    for (opcode, helper_stores) in cases {
        for target in [0x20000u32, 0x20003] {
            let run = run_decoded(
                &opcode.to_le_bytes(),
                0x3800_001f,
                |engine| {
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    let data: Vec<_> = (0..64u32).flat_map(|index| (target + index * 4).to_le_bytes()).collect();
                    engine.mem_write(0x20000, &data).unwrap();
                    engine.reg_write(ArmRegister::R0, 0x20040);
                    engine.reg_write(ArmRegister::R1, 0x1122_3344);
                    engine.reg_write(ArmRegister::R2, 0x18);
                    engine.reg_write(ArmRegister::R4, 0x18);
                    engine.reg_write(ArmRegister::R5, 0xaabb_ccdd);
                },
                0x1004,
                1,
                1,
            );
            assert_eq!(run.retired, 1, "opcode={opcode:#x}");
            assert_eq!(run.result.unwrap().instructions_executed, 1);
            assert_eq!(run.helper_stores, helper_stores, "opcode={opcode:#x}");
            assert!(run.handoffs.is_empty());
        }
    }
}

#[test]
fn decoded_memory_handoffs_precede_all_instruction_effects() {
    for (opcode, address, fault, generated) in [
        (0xe8b6_0036u32, 0x30000, Some(0x3000c), 1), // all words absent
        (0xe8b6_0036, 0x2fff4, Some(0x30000), 1),    // last word absent
        (0xe8b6_0036, 0x2fff8, Some(0x30004), 1),
        (0xe8a6_0036, 0x2fffc, Some(0x30008), 1), // stores partially complete in the CPU
        (0xe8a6_0036, 0x30000, Some(0x3000c), 1),
        (0xe8b6_0036, 0x20001, None, 2), // CPU rotation/alignment contract
        (0xe8a6_0036, 0x20001, None, 2),
        (0xe486_0004, 0x20001, None, 2),
        (0xe496_0004, 0x20001, None, 2),
        (0xe106_0092, 0x20001, None, 2), // unaligned SWP
        (0xe106_0092, 0x30000, Some(0x30000), 1),
        (0xe146_0092, 0x30001, Some(0x30001), 1),
        (0xe0c6_00d8, 0x30000, Some(0x30004), 1), // LDRD
        (0xe0c6_00f8, 0x30000, Some(0x30004), 1), // STRD
    ] {
        let code = [0xe5c9_8000, opcode, 0xe28a_a001]
            .into_iter()
            .flat_map(u32::to_le_bytes)
            .collect::<Vec<_>>();
        for interval in [2, 3] {
            let run = run_decoded(
                &code,
                0x3800_001f,
                |engine| {
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    engine.mem_write(0x20000, &0x1234_5678u32.to_le_bytes().repeat(64)).unwrap();
                    engine.mem_write(0x2ff00, &0x1122_3344u32.to_le_bytes().repeat(64)).unwrap();
                    for (register, value) in [
                        (0, 0x99aa_bbcc),
                        (1, 0x5566_7788),
                        (2, 0x1234_5678),
                        (4, 4),
                        (5, 5),
                        (6, address),
                        (8, 0x66),
                        (9, 0x20080),
                        (10, 41),
                    ] {
                        engine.cpu.reg_set(Mode::User, register, value);
                    }
                },
                0x100c,
                3,
                interval,
            );
            assert_eq!(run.retired, generated, "opcode={opcode:#x}, address={address:#x}");
            assert_eq!(run.helper_stores, 1, "faulting instruction must not store before handoff");
            assert_eq!(run.handoffs.len(), 1);
            let frame = &run.handoffs[0];
            assert_eq!(frame.executed, 1);
            assert_eq!(
                frame.regs,
                [
                    0x99aa_bbcc,
                    0x5566_7788,
                    0x1234_5678,
                    0,
                    4,
                    5,
                    address,
                    0,
                    0x66,
                    0x20080,
                    41,
                    0,
                    0,
                    0,
                    0,
                    0x1004
                ]
            );
            assert_eq!(frame.cpsr, 0x3800_001f);
            assert_eq!(frame.budget_remaining, 2);
            assert_eq!(frame.sample_remaining, interval - 1);
            if let Some(address) = fault {
                assert!(matches!(run.result, Err(WieError::InvalidMemoryAccess(actual)) if actual == address));
                assert_eq!(run.engine.reg_read(ArmRegister::SL), 41);
            } else {
                assert_eq!(run.result.unwrap().instructions_executed, 3);
                assert_eq!(run.engine.reg_read(ArmRegister::SL), 42);
            }
        }
    }
}

#[test]
fn decoded_thumb_multiple_handoffs_keep_partial_fault_state() {
    for (opcode, address, stack, fault) in [
        (0xce25u16, 0x2fffc, false, Some(0x30004)),
        (0xc625, 0x2fff8, false, Some(0x30000)),
        (0xbd25, 0x2fffc, true, Some(0x30008)),
        (0xb525, 0x30008, true, Some(0x30004)),
        (0xce25, 0x20001, false, None),
        (0xc625, 0x20001, false, None),
        (0xbc25, 0x20001, true, None),
        (0xb525, 0x20041, true, None),
    ] {
        let code = [0x703bu16, opcode, 0x3401].into_iter().flat_map(u16::to_le_bytes).collect::<Vec<_>>(); // strb r3,[r7]; transfer; add r4,#1
        let run = run_decoded(
            &code,
            0x3800_003f,
            |engine| {
                engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                engine.mem_write(0x20000, &0x1122_3344u32.to_le_bytes().repeat(64)).unwrap();
                engine.mem_write(0x2ff00, &0x1122_3344u32.to_le_bytes().repeat(64)).unwrap();
                engine.reg_write(ArmRegister::R0, 0x5566_7788);
                engine.reg_write(ArmRegister::R2, 0x99aa_bbcc);
                engine.reg_write(ArmRegister::R3, 0x66);
                engine.reg_write(ArmRegister::R4, 41);
                engine.reg_write(ArmRegister::R6, if stack { 0 } else { address });
                engine.reg_write(ArmRegister::SP, if stack { address } else { 0 });
                engine.reg_write(ArmRegister::R7, 0x20080);
                engine.reg_write(ArmRegister::LR, 0x1007);
            },
            0x1006,
            3,
            2,
        );
        assert_eq!(run.retired, if fault.is_some() { 1 } else { 2 });
        assert_eq!(run.helper_stores, 1);
        assert_eq!(run.handoffs.len(), 1);
        let frame = &run.handoffs[0];
        assert_eq!(frame.regs[15], 0x1002);
        assert_eq!(frame.regs[0], 0x5566_7788);
        assert_eq!(frame.regs[4], 41);
        assert_eq!(frame.regs[if stack { 13 } else { 6 }], address);
        assert_eq!(frame.executed, 1);
        if let Some(address) = fault {
            assert!(matches!(run.result, Err(WieError::InvalidMemoryAccess(actual)) if actual == address));
        } else {
            assert_eq!(run.result.unwrap().instructions_executed, 3);
        }
    }
}

#[test]
fn decoded_word_ranges_wrap_without_scalar_helpers() {
    for (opcode, words) in [(0xe8a6_0036u32, 4), (0xe8b6_0036, 4), (0xe0c6_00f8, 2), (0xe0c6_00d8, 2)] {
        for mapped in [false, true] {
            let code = [0xe5c9_8000, opcode].into_iter().flat_map(u32::to_le_bytes).collect::<Vec<_>>();
            let run = run_decoded(
                &code,
                0x3800_0010,
                |engine| {
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    if mapped {
                        // The public map range cannot represent the exclusive end 2^32.
                        engine.mem.pages[0xffff] = Some(MemoryPage {
                            bytes: Box::new([0x11; PAGE_SIZE]),
                            version: 0,
                        });
                    }
                    engine.mem_write(0, &0x2233_4455u32.to_le_bytes().repeat(2)).unwrap();
                    for (register, value) in [
                        (0, 0x1234_5678),
                        (1, 0x5566_7788),
                        (2, 0x99aa_bbcc),
                        (4, 4),
                        (5, 5),
                        (6, 0xffff_fff8),
                        (8, 0x66),
                        (9, 0x20080),
                    ] {
                        engine.cpu.reg_set(Mode::User, register, value);
                    }
                },
                0x1008,
                2,
                2,
            );
            assert_eq!(run.retired, if mapped { 2 } else { 1 });
            assert_eq!(run.helper_stores, 1);
            if mapped {
                assert_eq!(run.result.unwrap().instructions_executed, 2);
                assert!(run.handoffs.is_empty());
                assert_eq!(run.engine.reg_read(ArmRegister::R6), if words == 4 { 8 } else { 0 });
            } else {
                assert!(matches!(run.result, Err(WieError::InvalidMemoryAccess(0xffff_fffc))));
                assert_eq!(run.handoffs.len(), 1);
                assert_eq!(run.handoffs[0].regs[6], 0xffff_fff8);
                assert_eq!(run.handoffs[0].regs[15], 0x1004);
            }
        }
    }
}

#[test]
fn decoded_extended_operations_obey_budget_and_sample_boundaries() {
    let code = [0xe010_0190u32, 0xe032_2190, 0xe8a6_0007, 0xe1c6_40d4, 0xe10f_3000]
        .into_iter()
        .flat_map(u32::to_le_bytes)
        .collect::<Vec<_>>();
    for budget in 0..=6 {
        for interval in [1, 2, 3, 5, 6] {
            let run = run_decoded(
                &code,
                0x3800_0010,
                |engine| {
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    engine.mem_write(0x20000, &0x1122_3344u32.to_le_bytes().repeat(64)).unwrap();
                    engine.reg_write(ArmRegister::R0, 3);
                    engine.reg_write(ArmRegister::R1, 7);
                    engine.reg_write(ArmRegister::R2, 11);
                    engine.reg_write(ArmRegister::R6, 0x20040);
                },
                0x1014,
                budget,
                interval,
            );
            assert_eq!(run.retired, budget.min(5));
            assert_eq!(run.result.unwrap().instructions_executed, budget.min(5));
            assert_eq!(run.helper_stores, 0);
            assert!(run.handoffs.is_empty());
        }
    }
    for opcode in [0xf5d0_f013u32, 0xf550_ffff, 0xf7d0_f061] {
        let run = run_decoded(
            &opcode.to_le_bytes(),
            0x3800_0010,
            |engine| {
                engine.reg_write(ArmRegister::R0, 0xffff_ffff);
                engine.reg_write(ArmRegister::R1, 0xffff_ffff);
            },
            0x1004,
            1,
            1,
        );
        assert_eq!(run.retired, 1);
        assert_eq!(run.result.unwrap().instructions_executed, 1);
        assert_eq!(run.helper_stores, 0);
        assert!(run.handoffs.is_empty());
    }
}

#[test]
fn generated_arm_and_thumb_loops_match_interpreter_state_and_samples() {
    for thumb in [false, true] {
        let code = if thumb {
            [0x3001u16, 0x2803, 0xd1fc, 0x4770]
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect::<Vec<_>>()
        } else {
            [0xe2800001u32, 0xe3500003, 0x1afffffc, 0xe12fff1e]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect::<Vec<_>>()
        };
        for (end, budget, interval) in [(0x1000, 0, 1), (0x2000, 0, 1), (0x2000, 1, 1), (0x2000, 100, 2), (0x1004, 100, 4)] {
            let mut interpreter = Arm32CpuEngine::new();
            let mut compiled = Arm32CpuEngine::new();
            let cpsr = 0x1f | if thumb { 0x20 } else { 0 };
            for engine in [&mut interpreter, &mut compiled] {
                engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, &code).unwrap();
                engine.reg_write(ArmRegister::Cpsr, cpsr);
                engine.reg_write(ArmRegister::PC, 0x1000);
                engine.reg_write(ArmRegister::LR, 0x2000);
                engine.sampler.remaining = interval;
                engine.set_profiling(true);
            }
            let calls = Arc::new(Mutex::new(0));
            let mut jit = Jit::new(
                1,
                Box::new(NodeExecutor {
                    calls: calls.clone(),
                    ..NodeExecutor::default()
                }),
            );
            let sampled_key = RegionKey {
                pc: 0x1000,
                thumb,
                cpu_mode: 0x1f,
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            let expected = interpreter.run(end, budget, None).unwrap();
            let actual = compiled.run(end, budget, None).unwrap();
            assert_eq!(actual.instructions_executed, expected.instructions_executed);
            assert_eq!(
                matches!(actual.stop_reason, EngineStopReason::End),
                matches!(expected.stop_reason, EngineStopReason::End)
            );
            for index in 0..=reg::CPSR {
                assert_eq!(
                    compiled.cpu.reg_get(Mode::User, index),
                    interpreter.cpu.reg_get(Mode::User, index),
                    "register {index}, thumb={thumb}"
                );
            }
            assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
            assert_eq!(compiled.sampler.sequence, interpreter.sampler.sequence);
            let actual: Vec<_> = compiled
                .take_profile(true)
                .into_iter()
                .map(|sample| (sample.stack, sample.count))
                .collect();
            let expected: Vec<_> = interpreter
                .take_profile(true)
                .into_iter()
                .map(|sample| (sample.stack, sample.count))
                .collect();
            assert_eq!(actual, expected);
            assert_eq!(*calls.lock() > 0, budget > 0);
        }
    }
}

#[test]
fn arm_immediate_logical_flags_preserve_carry_when_rotation_is_zero() {
    for (opcode, rotated_carry) in [
        (0xe3b0_0002u32, None),
        (0xe23e_5080, None),
        (0xe3b0_0102, Some(true)),
        (0xe3b0_0202, Some(false)),
    ] {
        for carry in [false, true] {
            let cpsr = 0x1000_001f | (u32::from(carry) << 29);
            let mut interpreter = Arm32CpuEngine::new();
            let mut compiled = Arm32CpuEngine::new();
            for engine in [&mut interpreter, &mut compiled] {
                engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, &opcode.to_le_bytes()).unwrap();
                engine.reg_write(ArmRegister::Cpsr, cpsr);
                engine.reg_write(ArmRegister::PC, 0x1000);
                engine.reg_write(ArmRegister::LR, 1);
            }
            let calls = Arc::new(Mutex::new(0));
            let mut jit = Jit::new(
                1,
                Box::new(NodeExecutor {
                    calls: calls.clone(),
                    ..NodeExecutor::default()
                }),
            );
            let sampled_key = RegionKey {
                pc: 0x1000,
                thumb: false,
                cpu_mode: 0x1f,
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            compiled.run(0x1004, 1, None).unwrap();
            interpreter.run(0x1004, 1, None).unwrap();
            assert_eq!(*calls.lock(), 1);
            let expected_carry = u32::from(rotated_carry.unwrap_or(carry)) << 29;
            assert_eq!(compiled.reg_read(ArmRegister::Cpsr) & (1 << 29), expected_carry);
            assert_eq!(
                interpreter.reg_read(ArmRegister::Cpsr) & (1 << 29),
                expected_carry,
                "opcode={opcode:#x}, carry={carry}"
            );
            for index in 0..=reg::CPSR {
                assert_eq!(compiled.cpu.reg_get(Mode::User, index), interpreter.cpu.reg_get(Mode::User, index));
            }
        }
    }
}

#[test]
fn decoded_thumb_register_operations_match_interpreter_flags() {
    for kind in 0..16 {
        for right in [0u32, 1, 31, 32, 33, 255, 0xffff_ffff] {
            let code = (0x4008u16 | (kind << 6)).to_le_bytes();
            let mut interpreter = Arm32CpuEngine::new();
            let mut compiled = Arm32CpuEngine::new();
            for engine in [&mut interpreter, &mut compiled] {
                engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x1000, &code).unwrap();
                engine.reg_write(ArmRegister::Cpsr, 0xf000_003f);
                engine.reg_write(ArmRegister::PC, 0x1000);
                engine.reg_write(ArmRegister::R0, 0x8000_0001);
                engine.reg_write(ArmRegister::R1, right);
            }
            let calls = Arc::new(Mutex::new(0));
            let retired = Arc::new(Mutex::new(0));
            let mut jit = Jit::new(
                1,
                Box::new(NodeExecutor {
                    calls: calls.clone(),
                    retired: retired.clone(),
                    ..NodeExecutor::default()
                }),
            );
            let sampled_key = RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            interpreter.run(0x1002, 1, None).unwrap();
            compiled.run(0x1002, 1, None).unwrap();
            assert_eq!(*calls.lock(), 1);
            assert_eq!(*retired.lock(), 1);
            for index in 0..=reg::CPSR {
                assert_eq!(
                    compiled.cpu.reg_get(Mode::User, index),
                    interpreter.cpu.reg_get(Mode::User, index),
                    "op {kind}, right={right:#x}, reg {index}"
                );
            }
        }
    }
}

#[test]
fn decoded_arm_multiply_retires_in_wasm_with_matching_flags() {
    for opcode in [0xe0000190u32, 0xe0100190, 0x00100190, 0x10100190] {
        for (left, right) in [(0, 1), (u32::MAX, u32::MAX), (0x8000_0000, 2), (0x7fff_ffff, 3)] {
            for cpu_mode in [0x10, 0x1f] {
                let mut interpreter = Arm32CpuEngine::new();
                let mut compiled = Arm32CpuEngine::new();
                for engine in [&mut interpreter, &mut compiled] {
                    engine.mem_map(0x1000, 4, MemoryPermission::ReadWriteExecute);
                    engine.mem_write(0x1000, &opcode.to_le_bytes()).unwrap();
                    engine.reg_write(ArmRegister::Cpsr, 0x3800_0000 | u32::from(cpu_mode));
                    engine.reg_write(ArmRegister::PC, 0x1000);
                    engine.reg_write(ArmRegister::R0, left);
                    engine.reg_write(ArmRegister::R1, right);
                    engine.sampler.remaining = 1;
                    engine.set_profiling(true);
                }
                let retired = Arc::new(Mutex::new(0));
                let mut jit = Jit::new(
                    1,
                    Box::new(NodeExecutor {
                        retired: retired.clone(),
                        ..NodeExecutor::default()
                    }),
                );
                let sampled_key = RegionKey {
                    pc: 0x1000,
                    thumb: false,
                    cpu_mode,
                };
                jit.sample(sampled_key, Some((sampled_key, 8)));
                jit.maintain(&compiled.mem);
                compiled.jit = Some(jit);
                let expected = interpreter.run(0x1004, 1, None).unwrap();
                let actual = compiled.run(0x1004, 1, None).unwrap();
                assert_eq!(*retired.lock(), 1, "opcode={opcode:#x}");
                assert_eq!(actual.instructions_executed, expected.instructions_executed);
                for index in 0..=reg::CPSR {
                    assert_eq!(
                        compiled.cpu.reg_get(Mode::User, index),
                        interpreter.cpu.reg_get(Mode::User, index),
                        "opcode={opcode:#x}, left={left:#x}, right={right:#x}, register={index}"
                    );
                }
                let samples = compiled.take_profile(true);
                assert_eq!(samples.len(), 1);
                assert_eq!(samples[0].stack, [0x1000]);
                assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
                assert_eq!(compiled.sampler.sequence, interpreter.sampler.sequence);
            }
        }
    }
}

#[test]
fn decoded_memory_accesses_match_interpreter_memory_and_writeback() {
    let thumb = [
        0x6008u16, 0x6808, 0x7008, 0x7808, 0x8008, 0x8808, 0x5088, 0x5288, 0x5488, 0x5688, 0x5888, 0x5a88, 0x5c88, 0x5e88,
    ];
    let arm = [
        0xe5810004u32,
        0xe5910004,
        0xe5c10004,
        0xe5d10004,
        0xe4810004,
        0xe4910004,
        0xe5a10004,
        0xe5b10004,
        0xe5010004,
        0xe5110004,
        0xe1c100b0,
        0xe1d100b0,
        0xe1d100d0,
        0xe1d100f0,
    ];
    let programs = thumb
        .into_iter()
        .map(|raw| (true, raw.to_le_bytes().to_vec()))
        .chain(arm.into_iter().map(|raw| (false, raw.to_le_bytes().to_vec())));
    for (thumb, code) in programs {
        let run = run_decoded(
            &code,
            0xf000_001f | if thumb { 0x20 } else { 0 },
            |engine| {
                engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                engine.mem_write(0x20000, &[0xff, 0x80, 0x22, 0x11].repeat(64)).unwrap();
                engine.reg_write(ArmRegister::R0, 0x9988_7766);
                engine.reg_write(ArmRegister::R1, 0x20010);
                engine.reg_write(ArmRegister::R2, 4);
            },
            0x1000 + code.len() as u32,
            1,
            1,
        );
        assert_eq!(run.retired, 1, "{code:x?}");
        assert_eq!(run.result.unwrap().instructions_executed, 1);
        assert!(run.handoffs.is_empty());
    }
}

#[test]
fn memory_handoff_preserves_retired_prefix_and_fault_state() {
    for opcode in [0xe4810004u32, 0xe4910004, 0xe1c100b0, 0xe1d100b0] {
        for address in [0x20001, 0x30000] {
            for interval in [2, 3] {
                // strb r0, [r3]; load/store through r1; add r2, r2, #1
                let code = [0xe5c30000, opcode, 0xe2822001]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect::<Vec<_>>();
                let mut interpreter = Arm32CpuEngine::new();
                let mut compiled = Arm32CpuEngine::new();
                for engine in [&mut interpreter, &mut compiled] {
                    engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
                    engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
                    engine.mem_write(0x1000, &code).unwrap();
                    engine.mem_write(0x20000, &[0xff, 0x80, 0x22, 0x11].repeat(64)).unwrap();
                    engine.reg_write(ArmRegister::Cpsr, 0xf000_001f);
                    engine.reg_write(ArmRegister::PC, 0x1000);
                    engine.reg_write(ArmRegister::R0, 0x9988_7766);
                    engine.reg_write(ArmRegister::R1, address);
                    engine.reg_write(ArmRegister::R2, 41);
                    engine.reg_write(ArmRegister::R3, 0x20080);
                    engine.sampler.remaining = interval;
                    engine.set_profiling(true);
                }
                let calls = Arc::new(Mutex::new(0));
                let mut jit = Jit::new(
                    1,
                    Box::new(NodeExecutor {
                        calls: calls.clone(),
                        ..NodeExecutor::default()
                    }),
                );
                let sampled_key = RegionKey {
                    pc: 0x1000,
                    thumb: false,
                    cpu_mode: 0x1f,
                };
                jit.sample(sampled_key, Some((sampled_key, 8)));
                jit.maintain(&compiled.mem);
                compiled.jit = Some(jit);
                let expected = interpreter.run(0x100c, 8, None);
                let actual = compiled.run(0x100c, 8, None);
                assert_eq!(*calls.lock(), if address == 0x30000 { 1 } else { 2 });
                if address == 0x30000 {
                    assert!(matches!(expected, Err(WieError::InvalidMemoryAccess(0x30000))));
                    assert!(matches!(actual, Err(WieError::InvalidMemoryAccess(0x30000))));
                    assert_eq!(compiled.reg_read(ArmRegister::R2), 41);
                } else {
                    let expected = expected.unwrap();
                    let actual = actual.unwrap();
                    assert_eq!(actual.instructions_executed, 3);
                    assert_eq!(actual.instructions_executed, expected.instructions_executed);
                    assert!(matches!(actual.stop_reason, EngineStopReason::End));
                    assert!(matches!(expected.stop_reason, EngineStopReason::End));
                    assert_eq!(compiled.reg_read(ArmRegister::R2), 42);
                }
                for index in 0..=reg::CPSR {
                    assert_eq!(
                        compiled.cpu.reg_get(Mode::User, index),
                        interpreter.cpu.reg_get(Mode::User, index),
                        "opcode={opcode:#x}, address={address:#x}, interval={interval}, register={index}"
                    );
                }
                let mut actual = [0; 256];
                let mut expected = [0; 256];
                compiled.mem_read(0x20000, 256, &mut actual).unwrap();
                interpreter.mem_read(0x20000, 256, &mut expected).unwrap();
                assert_eq!(actual, expected);
                assert_eq!(actual[0x80], 0x66);
                assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
                assert_eq!(compiled.sampler.sequence, interpreter.sampler.sequence);
                let actual: Vec<_> = compiled
                    .take_profile(true)
                    .into_iter()
                    .map(|sample| (sample.stack, sample.count))
                    .collect();
                let expected: Vec<_> = interpreter
                    .take_profile(true)
                    .into_iter()
                    .map(|sample| (sample.stack, sample.count))
                    .collect();
                assert_eq!(actual, expected);
            }
        }
    }
}

#[test]
fn generated_store_keeps_cached_code_until_explicit_instruction_cache_flush() {
    for flush in [0xee07_0f15u32, 0xee07_0f35] {
        // I-cache all / MVA
        let mut engine = Arm32CpuEngine::new();
        engine.mem_map(0x1000, 6, MemoryPermission::ReadWriteExecute);
        engine.mem_map(0x10000, 4, MemoryPermission::ReadWriteExecute);
        engine.mem_write(0x1000, &[0x08, 0x80, 0x01, 0x22, 0x70, 0x47]).unwrap(); // strh r0,[r1]; mov r2,#1; bx lr
        engine.mem_write(0x10000, &flush.to_le_bytes()).unwrap();
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1000);
        engine.reg_write(ArmRegister::LR, 0x2000);
        engine.reg_write(ArmRegister::R0, 0x2209); // replacement: mov r2,#9
        engine.reg_write(ArmRegister::R1, 0x1002);
        engine.sampler.remaining = 17;
        let executor = NodeExecutor::default();
        let calls = executor.calls.clone();
        let retired = executor.retired.clone();
        let helper_stores = executor.helper_stores.clone();
        let mut jit = Jit::new(1, Box::new(executor));
        let sampled_key = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        jit.sample(sampled_key, Some((sampled_key, 8)));
        jit.maintain(&engine.mem);
        engine.jit = Some(jit);
        let result = engine.run(0x2000, 10, None).unwrap();
        assert_eq!(result.instructions_executed, 3);
        assert!(matches!(result.stop_reason, EngineStopReason::End));
        assert_eq!(engine.reg_read(ArmRegister::R2), 1);
        let mut replacement = [0; 2];
        engine.mem_read(0x1002, 2, &mut replacement).unwrap();
        assert_eq!(replacement, 0x2209u16.to_le_bytes());

        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1002);
        engine.reg_write(ArmRegister::R2, 0);
        assert_eq!(engine.run(0x1004, 1, None).unwrap().instructions_executed, 1);
        assert_eq!(engine.reg_read(ArmRegister::R2), 1);
        assert_eq!(*retired.lock(), 4);
        assert_eq!(*calls.lock(), 2);

        engine.reg_write(ArmRegister::Cpsr, 0x1f);
        engine.reg_write(ArmRegister::PC, 0x10000);
        engine.reg_write(ArmRegister::R0, 0x1002);
        assert_eq!(engine.run(0x10004, 1, None).unwrap().instructions_executed, 1);
        assert_eq!(*calls.lock(), 2);
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1002);
        engine.reg_write(ArmRegister::R2, 0);
        let entry = RegionKey {
            pc: 0x1002,
            thumb: true,
            cpu_mode: 0x1f,
        };
        assert!(engine.jit.as_mut().unwrap().lookup(entry, &engine.mem).is_none());
        engine.jit.as_mut().unwrap().sample(entry, Some((entry, 8)));
        engine.maintain();
        assert_eq!(engine.run(0x1004, 1, None).unwrap().instructions_executed, 1);
        assert_eq!(engine.reg_read(ArmRegister::R2), 9);
        assert_eq!(*retired.lock(), 5);
        assert_eq!(*calls.lock(), 3);
        assert_eq!(*helper_stores.lock(), 1);
        assert_eq!(engine.sampler.remaining, 11);
    }
}

#[test]
fn a_retired_sample_is_published_before_the_next_pc_fault() {
    let mut engine = Arm32CpuEngine::new();
    engine.mem_map(0x1000, 2, MemoryPermission::ReadWriteExecute);
    engine.mem_write(0x1000, &[0x00, 0x47]).unwrap(); // bx r0
    engine.reg_write(ArmRegister::Cpsr, 0x3f);
    engine.reg_write(ArmRegister::PC, 0x1000);
    engine.reg_write(ArmRegister::R0, 8);
    engine.set_profiling(true);
    engine.sampler.remaining = 1;
    let calls = Arc::new(Mutex::new(0));
    let mut jit = Jit::new(
        1,
        Box::new(NodeExecutor {
            calls: calls.clone(),
            ..NodeExecutor::default()
        }),
    );
    let sampled_key = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    jit.sample(sampled_key, Some((sampled_key, 8)));
    jit.maintain(&engine.mem);
    engine.jit = Some(jit);
    assert!(matches!(engine.run(8, 1, None), Err(WieError::InvalidMemoryAccess(8))));
    assert_eq!(*calls.lock(), 1);
    let samples = engine.take_profile(true);
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].stack, [0x1000]);
    assert_eq!(samples[0].count, 1);
    assert_eq!(engine.sampler.sequence, 1);
    assert_eq!(engine.reg_read(ArmRegister::PC), 8);
}

#[test]
fn original_byte_copy_loop_runs_without_native_hooks() {
    let code = [0x7803u16, 0x700b, 0x1c13, 0x3001, 0x3101, 0x3a01, 0x2b00, 0xdcf7, 0x4770]
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let mut interpreter = Arm32CpuEngine::new();
    let mut compiled = Arm32CpuEngine::new();
    let source: Vec<_> = (1..=16).collect();
    for engine in [&mut interpreter, &mut compiled] {
        engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
        engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
        engine.mem_write(0x1000, &code).unwrap();
        engine.mem_write(0x20000, &source).unwrap();
        engine.reg_write(ArmRegister::Cpsr, 0x3f);
        engine.reg_write(ArmRegister::PC, 0x1000);
        engine.reg_write(ArmRegister::LR, 0x2000);
        engine.reg_write(ArmRegister::R0, 0x20000);
        engine.reg_write(ArmRegister::R1, 0x20080);
        engine.reg_write(ArmRegister::R2, 15);
        engine.sampler.remaining = 17;
        engine.set_profiling(true);
    }
    let calls = Arc::new(Mutex::new(0));
    let mut jit = Jit::new(
        1,
        Box::new(NodeExecutor {
            calls: calls.clone(),
            ..NodeExecutor::default()
        }),
    );
    let sampled_key = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    jit.sample(sampled_key, Some((sampled_key, 8)));
    jit.maintain(&compiled.mem);
    compiled.jit = Some(jit);
    let expected = interpreter.run(0x2000, 1000, None).unwrap();
    let actual = compiled.run(0x2000, 1000, None).unwrap();
    assert_eq!(actual.instructions_executed, 129);
    assert_eq!(actual.instructions_executed, expected.instructions_executed);
    assert!(*calls.lock() >= 2);
    let mut copied = [0; 16];
    compiled.mem_read(0x20080, 16, &mut copied).unwrap();
    assert_eq!(&copied, source.as_slice());
    for index in 0..=reg::CPSR {
        assert_eq!(compiled.cpu.reg_get(Mode::User, index), interpreter.cpu.reg_get(Mode::User, index));
    }
    let actual: Vec<_> = compiled
        .take_profile(true)
        .into_iter()
        .map(|sample| (sample.stack, sample.count))
        .collect();
    let expected: Vec<_> = interpreter
        .take_profile(true)
        .into_iter()
        .map(|sample| (sample.stack, sample.count))
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn compiled_loops_reach_deadlines_with_matching_retirement_and_samples() {
    for thumb in [false, true] {
        let code: Vec<_> = if thumb {
            [0x3001u16, 0xe7fd].into_iter().flat_map(u16::to_le_bytes).collect()
        } else {
            [0xe2800001u32, 0xeafffffd].into_iter().flat_map(u32::to_le_bytes).collect()
        };
        let [mut compiled, mut interpreter] = core::array::from_fn(|_| {
            let mut engine = Arm32CpuEngine::new();
            engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, &code).unwrap();
            engine.reg_write(ArmRegister::Cpsr, if thumb { 0x3f } else { 0x1f });
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.mark_entry();
            engine.set_profiling(true);
            engine
        });
        let executor = NodeExecutor::default();
        let retired = executor.retired.clone();
        let mut jit = Jit::new(1, Box::new(executor));
        let entry = RegionKey {
            pc: 0x1000,
            thumb,
            cpu_mode: 0x1f,
        };
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&compiled.mem);
        jit.poll(&compiled.mem);
        assert!(jit.lookup(entry, &compiled.mem).is_some());
        compiled.jit = Some(jit);

        let result = compiled
            .run(0x2000, u32::MAX, Some(Instant::now() + core::time::Duration::from_millis(8)))
            .unwrap();
        assert!(matches!(result.stop_reason, EngineStopReason::Deadline));
        assert_eq!(*retired.lock(), result.instructions_executed);
        if result.instructions_executed != 0 {
            assert!(compiled.sampler.sequence > 0);
        }
        let expected = interpreter.run(0x2000, result.instructions_executed, None).unwrap();
        assert!(matches!(expected.stop_reason, EngineStopReason::Yield));
        assert_eq!(compiled.cpu, interpreter.cpu);
        assert_eq!(compiled.entry_pc, interpreter.entry_pc);
        assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
        assert_eq!(compiled.sampler.sequence, interpreter.sampler.sequence);
        let samples = [compiled.take_profile(true), interpreter.take_profile(true)]
            .map(|samples| samples.into_iter().map(|sample| (sample.stack, sample.count)).collect::<Vec<_>>());
        assert_eq!(samples[0], samples[1]);
    }
}

#[test]
fn compiled_prefix_and_nested_svc_calls_preserve_results_and_samples() {
    use crate::{ArmCore, JumpTo};

    async fn nested(core: &mut ArmCore, values: &mut Arc<Mutex<Vec<u32>>>) -> Result<JumpTo> {
        let pc = core.read_pc_lr()?.0;
        let value = core.run_function(0x10001, &[0]).await?;
        values.lock().push(value);
        Ok(JumpTo(pc | 1))
    }

    let samples = Arc::new(Mutex::new(Vec::new()));
    let observed_samples = samples.clone();
    let mut core = ArmCore::new(false, Some(Box::new(move |batch| samples.lock().extend(batch)))).unwrap();
    let values = Arc::new(Mutex::new(Vec::new()));
    core.register_svc_handler(1, nested, &values).unwrap();
    let mut outer = [0xc0, 0x46].repeat(5000);
    outer.extend_from_slice(&[0x01, 0xdf, 0x70, 0x47]);
    core.load(&outer, 0x1000, outer.len()).unwrap();
    let mut inner_code = [0x01, 0x30].repeat(5000);
    inner_code.extend_from_slice(&[0x70, 0x47]);
    core.load(&inner_code, 0x10000, inner_code.len()).unwrap();
    let calls = Arc::new(Mutex::new(0));
    {
        let mut inner = core.inner.lock();
        let engine = inner.engine.as_any_mut().downcast_mut::<Arm32CpuEngine>().unwrap();
        engine.sampler.remaining = 5001;
        let mut jit = Jit::new(
            1,
            Box::new(NodeExecutor {
                calls: calls.clone(),
                ..NodeExecutor::default()
            }),
        );
        for pc in [0x1000, 0x10000] {
            let sampled_key = RegionKey {
                pc,
                thumb: true,
                cpu_mode: 0x1f,
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&engine.mem);
        }
        engine.jit = Some(jit);
    }
    assert_eq!(futures::executor::block_on(core.run_function::<u32>(0x1001, &[17])).unwrap(), 17);
    assert_eq!(*values.lock(), [5000]);
    assert!(*calls.lock() >= 2);
    core.shutdown();
    assert!(
        observed_samples
            .lock()
            .iter()
            .any(|sample| sample.stack[0] == 0x1000 + 10_000 && sample.count == 1)
    );
}
