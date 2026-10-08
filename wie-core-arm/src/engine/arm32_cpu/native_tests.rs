use alloc::{boxed::Box, sync::Arc, vec, vec::Vec};
use core::sync::atomic::{AtomicU32, Ordering};

use test_utils::TestNativeExecutor;

use super::{Arm32CpuEngine, ArmEngine, ArmRegister, EngineStopReason, MemoryPermission, RegionKey};

struct Engines {
    interpreted: Arm32CpuEngine,
    native: Arm32CpuEngine,
    retired: Arc<AtomicU32>,
}

impl Engines {
    fn new(bytes: &[u8]) -> Self {
        let executor = TestNativeExecutor::new();
        let retired = executor.retired.clone();
        let mut engines = Self {
            interpreted: Arm32CpuEngine::new(),
            native: Arm32CpuEngine::with_backend(Some(Box::new(executor))),
            retired,
        };
        for engine in [&mut engines.interpreted, &mut engines.native] {
            engine.mem_map(0x1000, bytes.len(), MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, bytes).unwrap();
            engine.mem_map(0x20000, 0x20000, MemoryPermission::ReadWrite);
        }
        engines.native.record_image(0x1000, bytes.len());
        let preparation = engines.native.begin_preparation().unwrap().unwrap();
        let artifact = futures::executor::block_on(preparation).unwrap();
        engines.native.finish_preparation(Ok(artifact));
        engines
    }

    fn reset(&mut self, pc: u32, cpsr: u32, registers: &[(ArmRegister, u32)]) {
        for engine in [&mut self.interpreted, &mut self.native] {
            for index in 0..15 {
                engine.cpu.reg_set(arm32_cpu::Mode::User, index, 0);
            }
            engine.reg_write(ArmRegister::Cpsr, cpsr);
            engine.reg_write(ArmRegister::PC, pc);
            engine.reg_write(ArmRegister::LR, 0x8000);
            for &(register, value) in registers {
                engine.reg_write(register, value);
            }
        }
        self.retired.store(0, Ordering::Relaxed);
    }

    fn compare_run(&mut self, end: u32, budget: u32) {
        let pc = self.native.reg_read(ArmRegister::PC);
        let expected = self.interpreted.run(end, budget);
        let actual = self.native.run(end, budget);
        for index in 0..=16 {
            assert_eq!(
                self.native.cpu.reg_get(arm32_cpu::Mode::User, index),
                self.interpreted.cpu.reg_get(arm32_cpu::Mode::User, index),
                "register {index}, pc={pc:#x}, end={end:#x}, budget={budget}"
            );
        }
        match (actual, expected) {
            (Ok(actual), Ok(expected)) => {
                assert_eq!(actual.budget_consumed, expected.budget_consumed);
                match (&actual.stop_reason, expected.stop_reason) {
                    (EngineStopReason::End, EngineStopReason::End) | (EngineStopReason::Yield, EngineStopReason::Yield) => {}
                    (
                        EngineStopReason::Svc { category, lr, spsr },
                        EngineStopReason::Svc {
                            category: ec,
                            lr: el,
                            spsr: es,
                        },
                    ) => {
                        assert_eq!((*category, *lr, *spsr), (ec, el, es));
                    }
                    _ => panic!("different stop reasons"),
                }
            }
            (Err(actual), Err(expected)) => {
                assert_eq!(alloc::format!("{actual:?}"), alloc::format!("{expected:?}"));
            }
            _ => panic!("different execution outcomes"),
        }
    }

    fn compare_memory(&self) {
        for page in [0, 2, 3, 0xffff] {
            assert!(
                self.native.mem.pages[page].bytes == self.interpreted.mem.pages[page].bytes,
                "memory page {page:#x}"
            );
        }
    }
}

#[test]
fn native_memory_transfers_preserve_fault_state_and_completed_prefix() {
    let mut opcodes = vec![
        0xe581_2000,
        0xe591_2000,
        0xe5c1_2000,
        0xe5d1_2000,
        0xe1c1_20b0,
        0xe1d1_20b0,
        0xe1d1_20d0,
        0xe1d1_20f0,
        0xe101_2093,
        0xe141_2093,
        0xe101_2092,
        0xe581_f000,
        0xe5a1_2004,
        0xe481_2004,
        0xe1c2_20d0,
        0x0591_2000,
    ];
    for load in [0, 1] {
        for increment in [0, 1] {
            for before in [0, 1] {
                for write_back in [0, 1] {
                    opcodes.push(0xe801_003c | load << 20 | write_back << 21 | increment << 23 | before << 24);
                }
            }
        }
        for pre_index in [0, 1] {
            for subtract in [0, 1] {
                opcodes.push(0xe041_2094 | (if load == 1 { 2 } else { 3 }) << 5 | pre_index << 24 | (1 - subtract) << 23);
                opcodes.push(0xe401_2004 | load << 20 | pre_index << 24 | (1 - subtract) << 23);
            }
        }
    }
    opcodes.extend([0xe8a1_0006, 0xe891_0006]);
    // str r6,[r7] precedes every operation so a handoff must not replay completed writes.
    let bytes: Vec<_> = opcodes
        .iter()
        .flat_map(|opcode| [0xe587_6000, *opcode, 0xe12f_ff1e])
        .flat_map(u32::to_le_bytes)
        .collect();
    let mut engines = Engines::new(&bytes);
    for (index, opcode) in opcodes.into_iter().enumerate() {
        let pc = 0x1000 + index as u32 * 12;
        assert!(
            engines.native.aot.as_ref().unwrap().entries.contains_key(&RegionKey {
                pc: pc + 4,
                thumb: false,
                cpu_mode: 0x1f
            }),
            "{opcode:08x}"
        );
        for mapped_second in [false, true] {
            for address in [0x21000u32, 0x21004, 0x2fff8, 0x2fffc, 0x30000, 0x40000, 0x21001, 0xffff_fff8, 0xffff_fffc] {
                let mut double_unaligned = false;
                if opcode & 0x0e10_00d0 == 0x0000_00d0 {
                    let offset = ((opcode >> 4) & 0xf0) | (opcode & 15);
                    let effective = if opcode & 0x0100_0000 == 0 {
                        address
                    } else if opcode & 0x0080_0000 != 0 {
                        address.wrapping_add(offset)
                    } else {
                        address.wrapping_sub(offset)
                    };
                    double_unaligned = effective & 7 != 0;
                }
                for engine in [&mut engines.interpreted, &mut engines.native] {
                    engine.mem.pages[2].bytes.as_mut().unwrap().fill(0x55);
                    engine.mem.pages[3].bytes = mapped_second.then(|| Box::new([0x66; 0x10000]));
                    engine.mem.pages[0xffff].bytes = Some(Box::new([0x77; 0x10000]));
                }
                engines.reset(
                    pc,
                    0x9800_001f,
                    &[
                        (ArmRegister::R1, address),
                        (ArmRegister::R2, if opcode == 0xe1c2_20d0 { address } else { 0x89ab_cdef }),
                        (ArmRegister::R3, 0x7654_3210),
                        (ArmRegister::R4, 0x1122_3344),
                        (ArmRegister::R5, 0x5566_7788),
                        (ArmRegister::R6, 42),
                        (ArmRegister::R7, 0x23000),
                    ],
                );
                engines.compare_run(pc + 8, 2);
                engines.compare_memory();
                assert!(engines.retired.load(Ordering::Relaxed) >= 1, "{opcode:08x}");
                if matches!(address, 0x21000 | 0x21004) {
                    assert_eq!(
                        engines.retired.load(Ordering::Relaxed),
                        if double_unaligned { 1 } else { 2 },
                        "{opcode:08x}"
                    );
                }
            }
        }
    }
}

#[test]
fn native_branch_exchange_pc_loads_and_interpreter_handoffs_match() {
    let opcodes = [
        0xe1a0_f000u32, // mov pc,r0
        0xe12f_ff10,    // bx r0
        0xe12f_ff30,    // blx r0
        0xe591_f000,    // ldr pc,[r1]
        0xe491_f004,    // ldr pc,[r1],#4
        0xe8b1_8001,    // ldmia r1!,{r0,pc}
        0x0591_f000,    // ldreq pc,[r1]
    ];
    let bytes: Vec<_> = opcodes
        .into_iter()
        .flat_map(|opcode| [opcode, 0xe12f_ff1e])
        .flat_map(u32::to_le_bytes)
        .collect();
    let mut engines = Engines::new(&bytes);
    for index in 0..opcodes.len() {
        for target in [0x8000, 0x8001, 0x8002, 0x8003, 0xffc] {
            // ARMv5 declares unaligned ARM-state PC writes UNPREDICTABLE.
            if target & 3 == 2 || (index == 0 && target & 3 != 0) {
                continue;
            }
            let pc = 0x1000 + index as u32 * 8;
            engines.reset(pc, 0x1f, &[(ArmRegister::R0, target), (ArmRegister::R1, 0x21000)]);
            for engine in [&mut engines.interpreted, &mut engines.native] {
                engine.mem_write(0x21000, &target.to_le_bytes().repeat(2)).unwrap();
            }
            engines.compare_run(0x8000, 1);
            assert_eq!(engines.retired.load(Ordering::Relaxed), 1);
        }
    }
    // Thumb: str r0,[r1]; svc #4. The SVC remains interpreter-owned.
    let mut engines = Engines::new(&[0x08, 0x60, 0x04, 0xdf]);
    engines.reset(0x1001, 0x3f, &[(ArmRegister::R0, 42), (ArmRegister::R1, 0x21000)]);
    engines.compare_run(0x8000, 2);
    engines.compare_memory();
    assert_eq!(engines.retired.load(Ordering::Relaxed), 1);
}

#[test]
fn native_code_changes_only_after_explicit_instruction_cache_invalidation() {
    let mut engines = Engines::new(&[1, 0x20, 0x70, 0x47]); // movs r0,#1; bx lr
    for opcode in [0xee07_0f35u32, 0xee07_0f15] {
        engines.native.mem_write(0x1000, &[1, 0x20]).unwrap();
        engines.reset(0x1001, 0x3f, &[]);
        engines.native.run(0x8000, 2).unwrap();
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 1);
        assert_eq!(engines.retired.load(Ordering::Relaxed), 2);

        engines.native.mem_write(0x1000, &[2, 0x20]).unwrap();
        engines.reset(0x1001, 0x3f, &[]);
        engines.native.run(0x8000, 2).unwrap();
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 1);

        engines.native.mem_write(0x1800, &opcode.to_le_bytes()).unwrap();
        engines.reset(0x1800, 0x1f, &[(ArmRegister::R0, 0x1000)]);
        engines.native.run(0x1804, 1).unwrap();
        let key = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        assert!(!engines.native.aot.as_ref().unwrap().entries.contains_key(&key));
        engines.reset(0x1001, 0x3f, &[]);
        engines.native.run(0x8000, 2).unwrap();
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 2);
        assert_eq!(engines.retired.load(Ordering::Relaxed), 0);
        engines.native.aot.as_mut().unwrap().poll_recompilations();
        engines.reset(0x1001, 0x3f, &[]);
        engines.native.run(0x8000, 2).unwrap();
        assert_eq!(engines.native.reg_read(ArmRegister::R0), 2);
        assert_eq!(engines.retired.load(Ordering::Relaxed), 2);

        // Install the original bytes again before testing the next CP15 operation.
        engines.native.mem_write(0x1000, &[1, 0x20]).unwrap();
        engines.native.aot.as_mut().unwrap().invalidate(0x1000..0x1020);
        engines.reset(0x1001, 0x3f, &[]);
        engines.native.run(0x8000, 2).unwrap();
        engines.native.aot.as_mut().unwrap().poll_recompilations();
    }
}

#[test]
fn native_arithmetic_conditions_and_shifts_match_interpreter() {
    let mut opcodes = Vec::new();
    for op in 0..16 {
        let rd = if (8..=11).contains(&op) { 0 } else { 2 };
        opcodes.push(0xe010_0001 | op << 21 | rd << 12);
    }
    for kind in 0..4 {
        for immediate in [0, 1, 31] {
            opcodes.push(0xe1b0_2001 | kind << 5 | immediate << 7);
        }
        opcodes.push(0xe1b0_2311 | kind << 5);
    }
    opcodes.extend([0xe3b0_2102, 0xe012_0190, 0xe032_4190, 0xe16f_2f11, 0xe10f_2000, 0xe128_f001]);
    for signed in [0, 1] {
        for accumulate in [0, 1] {
            opcodes.push(0xe093_2190 | signed << 22 | accumulate << 21);
        }
    }
    for condition in 0..14 {
        opcodes.push(condition << 28 | 0x0280_2001);
    }
    let bytes: Vec<_> = opcodes
        .iter()
        .flat_map(|opcode| [*opcode, 0xe12f_ff1e])
        .flat_map(u32::to_le_bytes)
        .collect();
    let mut engines = Engines::new(&bytes);
    for (index, opcode) in opcodes.into_iter().enumerate() {
        let pc = 0x1000 + index as u32 * 8;
        assert!(
            engines.native.aot.as_ref().unwrap().entries.contains_key(&RegionKey {
                pc,
                thumb: false,
                cpu_mode: 0x1f
            }),
            "{opcode:08x}"
        );
        for left in [0, 1, 0x7fff_ffff, 0x8000_0000, 0xffff_ffff] {
            for right in [0, 1, 0x8000_0001, 0xffff_ffff] {
                for flags in (0..16).map(|flags| flags << 28) {
                    for amount in [0, 1, 31, 32, 33, 255, 256] {
                        if opcode & 0x0fff_0f10 != 0x01b0_0310 && amount != 0 {
                            continue;
                        }
                        engines.reset(
                            pc,
                            flags | 0x0800_001f,
                            &[
                                (ArmRegister::R0, left),
                                (ArmRegister::R1, right),
                                (ArmRegister::R2, 0x7654_3210),
                                (ArmRegister::R3, amount),
                                (ArmRegister::R4, 0xffff_ffff),
                            ],
                        );
                        engines.compare_run(pc + 4, 1);
                        assert_eq!(engines.retired.load(Ordering::Relaxed), 1, "{opcode:08x}");
                    }
                }
            }
        }
    }
}

#[test]
fn native_loops_interior_entries_and_budgets_match_interpreter() {
    // subs r0,#1; bne 0x1000; bx lr; b 0x1006
    let bytes: Vec<_> = [0x3801u16, 0xd1fd, 0x4770, 0xe7fe].into_iter().flat_map(u16::to_le_bytes).collect();
    let mut engines = Engines::new(&bytes);
    for pc in [0x1000, 0x1002, 0x1004, 0x1006] {
        for end in [0x1000, 0x1002, 0x1004, 0x1006, 0x8000] {
            engines.reset(pc | 1, 0x3f, &[(ArmRegister::R0, 3)]);
            for budget in [0, 1, 2, 9] {
                engines.compare_run(end, budget);
            }
        }
    }
    engines.reset(0x1007, 0x3f, &[]);
    engines.compare_run(0x8000, 100);
    assert_eq!(engines.retired.load(Ordering::Relaxed), 100);

    for suffix in [0xf802u16, 0xe802] {
        let bytes: Vec<_> = [0xf000, suffix, 0x3001, 0x4770, 0x3101, 0x4770]
            .into_iter()
            .flat_map(u16::to_le_bytes)
            .collect();
        let mut engines = Engines::new(&bytes);
        engines.reset(0x1001, 0x3f, &[]);
        engines.compare_run(0x1008, 1);
    }
}
