extern crate std;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use alloc::{collections::BTreeMap, string::String, sync::Arc};

use serde::Deserialize;
use spin::Mutex;
use wie_arm_jit::{Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion};

use super::*;

#[derive(Deserialize)]
struct NodeResult {
    frame: [u32; 23],
    effects: Vec<NodeEffect>,
    exit: u32,
}

#[derive(Deserialize)]
enum NodeEffect {
    Sample([u32; 3]),
    Store([u32; 3]),
}

struct NodeModule {
    bytes: Vec<u8>,
    source: Vec<Vec<CodePageStamp>>,
}

struct NodeExecutor {
    modules: BTreeMap<u64, NodeModule>,
    completion: Option<CompileCompletion>,
    calls: Arc<Mutex<u32>>,
}

impl CompiledExecutor for NodeExecutor {
    fn submit(&mut self, request: CompileRequest) -> Admission {
        let artifact = wie_arm_wasm::compile(&request).unwrap();
        let size = artifact.bytes.len();
        self.modules.insert(
            request.request,
            NodeModule {
                bytes: artifact.bytes,
                source: request.regions.iter().map(|region| region.source.clone()).collect(),
            },
        );
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
        for base in [0x1000, 0x20000] {
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
            "wasm": self.modules[&handle.generation].bytes,
            "source": self.modules[&handle.generation].source[handle.slot as usize],
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
                NodeEffect::Sample([pc, cpsr, r7]) => access.sample_prepare(pc, cpsr, r7),
                NodeEffect::Store([address, width, value]) => {
                    assert!(matches!(
                        access.store(address, width, value),
                        AccessResult::Complete(0) | AccessResult::Invalidated
                    ));
                }
            }
        }
        *frame = bytemuck::pod_read_unaligned(bytemuck::cast_slice(&result.frame));
        Ok(match result.exit {
            0 => CompiledExit::Dispatch,
            1 => CompiledExit::Sample,
            2 => CompiledExit::Budget,
            3 => CompiledExit::End,
            4 => CompiledExit::InterpretOne,
            5 => CompiledExit::Invalidated,
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
                    modules: BTreeMap::new(),
                    completion: None,
                    calls: calls.clone(),
                }),
            );
            jit.sample(
                RegionKey {
                    pc: 0x1000,
                    thumb,
                    cpu_mode: 0x1f,
                },
                8,
            );
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            let expected = interpreter.run(end, budget).unwrap();
            let actual = compiled.run(end, budget).unwrap();
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
                    modules: BTreeMap::new(),
                    completion: None,
                    calls: calls.clone(),
                }),
            );
            jit.sample(
                RegionKey {
                    pc: 0x1000,
                    thumb: false,
                    cpu_mode: 0x1f,
                },
                8,
            );
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            compiled.run(0x1004, 1).unwrap();
            interpreter.run(0x1004, 1).unwrap();
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
    for kind in (0..16).filter(|kind| *kind != 13) {
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
            let mut jit = Jit::new(
                1,
                Box::new(NodeExecutor {
                    modules: BTreeMap::new(),
                    completion: None,
                    calls: calls.clone(),
                }),
            );
            jit.sample(
                RegionKey {
                    pc: 0x1000,
                    thumb: true,
                    cpu_mode: 0x1f,
                },
                8,
            );
            jit.maintain(&compiled.mem);
            compiled.jit = Some(jit);
            interpreter.run(0x1002, 1).unwrap();
            compiled.run(0x1002, 1).unwrap();
            assert_eq!(*calls.lock(), 1);
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
        let mut interpreter = Arm32CpuEngine::new();
        let mut compiled = Arm32CpuEngine::new();
        for engine in [&mut interpreter, &mut compiled] {
            engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
            engine.mem_map(0x20000, 256, MemoryPermission::ReadWrite);
            engine.mem_write(0x1000, &code).unwrap();
            engine.mem_write(0x20000, &[0xff, 0x80, 0x22, 0x11].repeat(64)).unwrap();
            engine.reg_write(ArmRegister::Cpsr, 0xf000_001f | if thumb { 0x20 } else { 0 });
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.reg_write(ArmRegister::R0, 0x9988_7766);
            engine.reg_write(ArmRegister::R1, 0x20010);
            engine.reg_write(ArmRegister::R2, 4);
            engine.sampler.remaining = 1;
            engine.set_profiling(true);
        }
        let calls = Arc::new(Mutex::new(0));
        let mut jit = Jit::new(
            1,
            Box::new(NodeExecutor {
                modules: BTreeMap::new(),
                completion: None,
                calls: calls.clone(),
            }),
        );
        jit.sample(
            RegionKey {
                pc: 0x1000,
                thumb,
                cpu_mode: 0x1f,
            },
            8,
        );
        jit.maintain(&compiled.mem);
        compiled.jit = Some(jit);
        interpreter.run(0x1000 + code.len() as u32, 1).unwrap();
        compiled.run(0x1000 + code.len() as u32, 1).unwrap();
        assert_eq!(*calls.lock(), 1, "{code:x?}");
        for index in 0..=reg::CPSR {
            assert_eq!(
                compiled.cpu.reg_get(Mode::User, index),
                interpreter.cpu.reg_get(Mode::User, index),
                "{code:x?}, reg {index}"
            );
        }
        let mut actual = [0; 256];
        let mut expected = [0; 256];
        compiled.mem_read(0x20000, 256, &mut actual).unwrap();
        interpreter.mem_read(0x20000, 256, &mut expected).unwrap();
        assert_eq!(actual, expected, "{code:x?}");
        let samples = compiled.take_profile(true);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stack, [0x1000]);
        assert_eq!(compiled.sampler.remaining, interpreter.sampler.remaining);
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
                        modules: BTreeMap::new(),
                        completion: None,
                        calls: calls.clone(),
                    }),
                );
                jit.sample(
                    RegionKey {
                        pc: 0x1000,
                        thumb: false,
                        cpu_mode: 0x1f,
                    },
                    8,
                );
                jit.maintain(&compiled.mem);
                compiled.jit = Some(jit);
                let expected = interpreter.run(0x100c, 8);
                let actual = compiled.run(0x100c, 8);
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
fn generated_store_stops_before_executing_the_instruction_it_replaced() {
    let mut engine = Arm32CpuEngine::new();
    engine.mem_map(0x1000, 6, MemoryPermission::ReadWriteExecute);
    engine.mem_write(0x1000, &[0x08, 0x80, 0x01, 0x22, 0x70, 0x47]).unwrap(); // strh r0, [r1]; mov r2, #1; bx lr
    engine.reg_write(ArmRegister::Cpsr, 0x3f);
    engine.reg_write(ArmRegister::PC, 0x1000);
    engine.reg_write(ArmRegister::LR, 0x2000);
    engine.reg_write(ArmRegister::R0, 0x2209); // mov r2, #9
    engine.reg_write(ArmRegister::R1, 0x1002);
    let calls = Arc::new(Mutex::new(0));
    let mut jit = Jit::new(
        1,
        Box::new(NodeExecutor {
            modules: BTreeMap::new(),
            completion: None,
            calls: calls.clone(),
        }),
    );
    jit.sample(
        RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        },
        8,
    );
    jit.maintain(&engine.mem);
    engine.jit = Some(jit);
    let result = engine.run(0x2000, 10).unwrap();
    assert_eq!(result.instructions_executed, 3);
    assert!(matches!(result.stop_reason, EngineStopReason::End));
    assert_eq!(engine.reg_read(ArmRegister::R2), 9);
    assert_eq!(*calls.lock(), 1);
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
            modules: BTreeMap::new(),
            completion: None,
            calls: calls.clone(),
        }),
    );
    jit.sample(
        RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        },
        8,
    );
    jit.maintain(&engine.mem);
    engine.jit = Some(jit);
    assert!(matches!(engine.run(8, 1), Err(WieError::InvalidMemoryAccess(8))));
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
            modules: BTreeMap::new(),
            completion: None,
            calls: calls.clone(),
        }),
    );
    jit.sample(
        RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        },
        8,
    );
    jit.maintain(&compiled.mem);
    compiled.jit = Some(jit);
    let expected = interpreter.run(0x2000, 1000).unwrap();
    let actual = compiled.run(0x2000, 1000).unwrap();
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
fn compiled_prefix_and_nested_svc_calls_share_the_scheduler_budget() {
    use crate::{ArmCore, JumpTo};
    use core::{
        pin::pin,
        task::{Context, Poll, Waker},
    };

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
                modules: BTreeMap::new(),
                completion: None,
                calls: calls.clone(),
            }),
        );
        for pc in [0x1000, 0x10000] {
            jit.sample(
                RegionKey {
                    pc,
                    thumb: true,
                    cpu_mode: 0x1f,
                },
                8,
            );
            jit.maintain(&engine.mem);
        }
        engine.jit = Some(jit);
    }
    let observer = core.clone();
    let mut run = pin!(core.run_function::<u32>(0x1001, &[17]));
    let mut cx = Context::from_waker(Waker::noop());
    assert!(run.as_mut().poll(&mut cx).is_pending());
    assert_eq!(observer.read_param(0).unwrap(), 4999);
    assert!(values.lock().is_empty());
    assert!(matches!(run.as_mut().poll(&mut cx), Poll::Ready(Ok(17))));
    assert_eq!(*values.lock(), [5000]);
    assert!(*calls.lock() >= 2);
    observer.shutdown();
    assert!(
        observed_samples
            .lock()
            .iter()
            .any(|sample| sample.stack[0] == 0x1000 + 10_000 && sample.count == 1)
    );
}
