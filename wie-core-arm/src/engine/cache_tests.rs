use alloc::{collections::BTreeMap, collections::BTreeSet, collections::VecDeque, string::String, sync::Arc, vec};

use spin::Mutex;
use wie_arm_jit::{Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion};

use super::*;

#[derive(Default)]
struct Responses {
    requests: Vec<CompileRequest>,
    request_ir_sizes: Vec<usize>,
    region_ir_sizes: Vec<Vec<usize>>,
    admissions: VecDeque<Admission>,
    ready: VecDeque<CompileCompletion>,
    retired: Vec<CompiledHandle>,
    executed: Option<Vec<u32>>,
}

struct DeferredExecutor(Arc<Mutex<Responses>>);

impl CompiledExecutor for DeferredExecutor {
    fn submit(&mut self, request: &CompileRequest) -> Admission {
        let mut responses = self.0.lock();
        responses.request_ir_sizes.push(request.ir_size());
        responses
            .region_ir_sizes
            .push(request.regions.iter().map(|region| region.ir_size()).collect());
        responses.requests.push(request.clone());
        responses.admissions.pop_front().unwrap_or(Admission::Accepted)
    }

    fn poll(&mut self) -> Option<CompileCompletion> {
        self.0.lock().ready.pop_front()
    }

    fn execute(&mut self, _: CompiledHandle, frame: &mut RunFrame, _: &mut dyn ExecutionAccess) -> core::result::Result<CompiledExit, String> {
        self.0
            .lock()
            .executed
            .as_mut()
            .expect("cache tests do not execute a CPU")
            .push(frame.regs[15]);
        Ok(CompiledExit::InterpretOne)
    }

    fn retire(&mut self, handles: &[CompiledHandle]) {
        self.0.lock().retired.extend_from_slice(handles);
    }

    fn shutdown(&mut self) {
        self.0.lock().ready.clear();
    }
}

fn completion(request: &CompileRequest) -> CompileCompletion {
    CompileCompletion {
        session: request.session,
        request: request.request,
        result: Ok(CompiledArtifact {
            encoded_size: 100,
            regions: request
                .regions
                .iter()
                .enumerate()
                .map(|(slot, region)| CompiledRegion {
                    manifest: ManifestRegion {
                        entry: region.ir.entry,
                        source: region.source.clone(),
                        export: format!("region_{slot}"),
                        expected_old: region.expected_old,
                    },
                    handle: CompiledHandle {
                        slot: slot as u32,
                        generation: request.request,
                    },
                })
                .collect(),
        }),
    }
}

#[test]
fn retired_loop_samples_heat_the_observed_entry_without_replacing_profile_pcs() {
    for thumb in [false, true] {
        for profiling in [false, true] {
            for batch in [1, 127, 1024] {
                let responses = Arc::new(Mutex::new(Responses::default()));
                let mut engine = Arm32CpuEngine::new();
                let mut code = if thumb {
                    [0xc0, 0x46].repeat(63)
                } else {
                    0xe1a00000u32.to_le_bytes().repeat(63)
                };
                // The final branch returns to the first of the loop's 64 CPU instructions.
                code.extend_from_slice(if thumb { &[0xbf, 0xe7] } else { &[0xbf, 0xff, 0xff, 0xea] });
                engine.mem_map(0x10000, code.len(), MemoryPermission::ReadWriteExecute);
                engine.mem_write(0x10000, &code).unwrap();
                engine.reg_write(ArmRegister::Cpsr, if thumb { 0x3f } else { 0x1f });
                engine.reg_write(ArmRegister::PC, 0x10000);
                engine.set_profiling(profiling);
                engine.jit = Some(Jit::new(1, Box::new(DeferredExecutor(responses.clone()))));

                let before_sample = engine.run(0x20000, 1023, None).unwrap();
                assert_eq!(before_sample.instructions_executed, 1023);
                assert_eq!(engine.sampler.sequence, 0);
                engine.maintain();
                assert!(
                    responses.lock().requests.is_empty(),
                    "branch visits must not heat without a retired sample"
                );
                assert_eq!(engine.run(0x20000, 0, None).unwrap().instructions_executed, 0);
                assert_eq!(engine.sampler.sequence, 0);

                let mut retired = 1023;
                while retired < 12_000 {
                    let count = batch.min(12_000 - retired);
                    let result = engine.run(0x20000, count, None).unwrap();
                    assert!(matches!(result.stop_reason, EngineStopReason::Yield));
                    assert_eq!(result.instructions_executed, count);
                    retired += count;
                    engine.maintain();
                    if engine.sampler.sequence < 8 {
                        assert!(responses.lock().requests.is_empty(), "admission must wait for eight shared samples");
                    }
                }

                let samples = engine.take_profile(true);
                assert!(engine.sampler.sequence >= 8);
                if profiling {
                    assert!(samples.len() > 1, "jitter must still select distinct actual instruction PCs");
                    assert_eq!(samples.iter().map(|sample| sample.count).sum::<u64>(), engine.sampler.sequence);
                    assert!(
                        samples
                            .iter()
                            .all(|sample| { sample.stack.len() == 1 && (0x10000..0x10000 + code.len() as u32).contains(&sample.stack[0]) })
                    );
                } else {
                    assert!(samples.is_empty());
                }
                let requests = &responses.lock().requests;
                assert_eq!(requests.len(), 1, "thumb={thumb}, profiling={profiling}, batch={batch}");
                assert_eq!(requests[0].regions.len(), 1);
                assert_eq!(
                    requests[0].regions[0].ir.entry,
                    RegionKey {
                        pc: 0x10000,
                        thumb,
                        cpu_mode: 0x1f,
                    }
                );
            }
        }
    }
}

#[test]
fn interpreter_lookup_waits_for_a_transfer_and_rechecks_after_a_handoff() {
    for thumb in [false, true] {
        for branch in [false, true] {
            let responses = Arc::new(Mutex::new(Responses {
                executed: Some(Vec::new()),
                ..Responses::default()
            }));
            let mut engine = Arm32CpuEngine::new();
            let code: Vec<_> = if thumb {
                [0x46c0u16, if branch { 0xe001 } else { 0x46c0 }, 0x46c0, 0x46c0, 0x46c0, 0x4770]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect()
            } else {
                [
                    0xe1a00000u32,
                    if branch { 0xea000001 } else { 0xe1a00000 },
                    0xe1a00000,
                    0xe1a00000,
                    0xe1a00000,
                    0xe12fff1e,
                ]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect()
            };
            let width = if thumb { 2 } else { 4 };
            let entry = RegionKey {
                pc: 0x1000 + 4 * width,
                thumb,
                cpu_mode: 0x1f,
            };
            engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x1000, &code).unwrap();
            engine.reg_write(ArmRegister::Cpsr, if thumb { 0x3f } else { 0x1f });
            engine.reg_write(ArmRegister::PC, 0x1000);
            engine.reg_write(ArmRegister::LR, 0x2000);
            let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
            jit.sample(entry, Some((entry, 8)));
            jit.maintain(&engine.mem);
            let request = responses.lock().requests[0].clone();
            responses.lock().ready.push_back(completion(&request));
            engine.jit = Some(jit);

            let result = engine.run(0x2000, 10, None).unwrap();
            assert!(matches!(result.stop_reason, EngineStopReason::End));
            assert_eq!(result.instructions_executed, if branch { 4 } else { 6 });
            assert_eq!(engine.reg_read(ArmRegister::PC), 0x2000);
            assert_eq!(engine.sampler.sequence, 0);
            assert_eq!(
                responses.lock().executed.as_ref().unwrap(),
                &if branch { vec![entry.pc, entry.pc + width] } else { Vec::new() },
                "thumb={thumb}, branch={branch}"
            );
        }
    }
}

#[test]
fn covered_instruction_entries_reuse_current_canonical_translations() {
    for thumb in [false, true] {
        for cpu_mode in [0x10, 0x1f] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
            let mut memory = EmulatedMemory::new();
            // nop; branch over two unreachable nops; add; svc; unreachable bx lr.
            let code: Vec<_> = if thumb {
                [0x46c0u16, 0xe001, 0x46c0, 0x46c0, 0x3001, 0xdf00, 0x4770]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect()
            } else {
                [0xe1a00000u32, 0xea000001, 0xe1a00000, 0xe1a00000, 0xe2800001, 0xef000000, 0xe12fff1e]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect()
            };
            let entry = RegionKey { pc: 0x1000, thumb, cpu_mode };
            let width = if thumb { 2 } else { 4 };
            memory.map(entry.pc, code.len());
            memory.write_range(entry.pc, &code).unwrap();
            jit.sample(entry, Some((entry, 8)));
            jit.maintain(&memory);
            let request = responses.lock().requests[0].clone();
            assert_eq!(
                request.regions[0]
                    .ir
                    .blocks
                    .iter()
                    .flat_map(|block| &block.instructions)
                    .map(|instruction| instruction.pc)
                    .collect::<Vec<_>>(),
                [entry.pc, entry.pc + width, entry.pc + 4 * width]
            );
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            let installed = jit.lookup(entry, &memory).unwrap();
            for offset in [width, 4 * width] {
                let interior = RegionKey {
                    pc: entry.pc + offset,
                    ..entry
                };
                assert_eq!(jit.lookup(interior, &memory), Some(installed), "thumb={thumb}, mode={cpu_mode:#x}");
                jit.sample(interior, Some((interior, 8)));
                jit.maintain(&memory);
            }
            assert_eq!(responses.lock().requests.len(), 1);
            for offset in [1, 2 * width, 3 * width, 5 * width, 6 * width] {
                assert!(
                    jit.lookup(
                        RegionKey {
                            pc: entry.pc + offset,
                            ..entry
                        },
                        &memory
                    )
                    .is_none()
                );
            }
            assert!(jit.lookup(RegionKey { thumb: !thumb, ..entry }, &memory).is_none());
            assert!(
                jit.lookup(
                    RegionKey {
                        cpu_mode: cpu_mode ^ 0x0f,
                        ..entry
                    },
                    &memory
                )
                .is_none()
            );
            assert!(
                jit.lookup(
                    RegionKey {
                        pc: entry.pc + width,
                        thumb: !thumb,
                        ..entry
                    },
                    &memory
                )
                .is_none()
            );
            assert!(
                jit.lookup(
                    RegionKey {
                        pc: entry.pc + width,
                        cpu_mode: cpu_mode ^ 0x0f,
                        ..entry
                    },
                    &memory
                )
                .is_none()
            );

            memory.write_range(entry.pc, &code).unwrap();
            let interior = RegionKey {
                pc: entry.pc + width,
                ..entry
            };
            assert!(jit.lookup(interior, &memory).is_none());
            assert!(jit.lookup(entry, &memory).is_none());
            assert_eq!(responses.lock().retired, [installed]);
            jit.sample(interior, Some((interior, 9)));
            jit.maintain(&memory);
            let replacement = responses.lock().requests[1].clone();
            assert_eq!(replacement.regions[0].ir.entry, interior);
            assert_eq!(replacement.regions[0].expected_old, None);
            responses.lock().ready.push_back(completion(&replacement));
            jit.poll(&memory);
            let current = jit.lookup(interior, &memory).unwrap();
            assert_ne!(current, installed);
            assert_eq!(
                jit.lookup(
                    RegionKey {
                        pc: entry.pc + 4 * width,
                        ..entry
                    },
                    &memory
                )
                .unwrap(),
                current
            );
            assert!(jit.lookup(entry, &memory).is_none());
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            assert_eq!(jit.lookup(interior, &memory).unwrap(), current);
            assert!(!responses.lock().retired.contains(&current));
            jit.shutdown();
            responses.lock().ready.push_back(completion(&replacement));
            jit.poll(&memory);
            for offset in [0, width, 4 * width] {
                assert!(
                    jit.lookup(
                        RegionKey {
                            pc: entry.pc + offset,
                            ..entry
                        },
                        &memory
                    )
                    .is_none()
                );
            }
        }
    }
}

#[test]
fn condition_false_unsupported_frontier_retires_once_in_the_dispatcher() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut engine = Arm32CpuEngine::new();
    // mov r0,#1; svceq #0; add r0,r0,#1
    let code: Vec<_> = [0xe3a00001u32, 0x0f000000, 0xe2800001].into_iter().flat_map(u32::to_le_bytes).collect();
    engine.mem_map(0x1000, code.len(), MemoryPermission::ReadWriteExecute);
    engine.mem_write(0x1000, &code).unwrap();
    let entry = RegionKey {
        pc: 0x1000,
        thumb: false,
        cpu_mode: 0x1f,
    };
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    jit.sample(entry, Some((entry, 8)));
    jit.maintain(&engine.mem);
    let request = responses.lock().requests[0].clone();
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&engine.mem);
    assert!(jit.lookup(entry, &engine.mem).is_some());
    assert!(jit.lookup(RegionKey { pc: 0x1004, ..entry }, &engine.mem).is_none());
    engine.jit = Some(jit);
    engine.reg_write(ArmRegister::Cpsr, 0x1f);
    engine.reg_write(ArmRegister::PC, 0x1004);
    engine.reg_write(ArmRegister::R0, 7);
    engine.sampler.profiling = true;
    engine.sampler.remaining = 1;

    let result = engine.run(0x1008, 1, None).unwrap();
    assert!(matches!(result.stop_reason, EngineStopReason::End));
    assert_eq!(result.instructions_executed, 1);
    assert_eq!(engine.reg_read(ArmRegister::PC), 0x1008);
    assert_eq!(engine.reg_read(ArmRegister::Cpsr), 0x1f);
    assert_eq!(engine.reg_read(ArmRegister::R0), 7);
    assert_eq!(engine.sampler.sequence, 1);
    let samples = engine.take_profile(true);
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].stack, [0x1004]);
    assert_eq!(samples[0].count, 1);
    let result = engine.run(0x100c, 1, None).unwrap();
    assert!(matches!(result.stop_reason, EngineStopReason::End));
    assert_eq!(result.instructions_executed, 1);
    assert_eq!(engine.reg_read(ArmRegister::PC), 0x100c);
    assert_eq!(engine.reg_read(ArmRegister::R0), 8);
}

#[test]
fn covered_candidates_do_not_duplicate_pending_compilation_or_starve_other_code() {
    for queued_before_submission in [false, true] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let entry = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let other = RegionKey { pc: 0x20000, ..entry };
        let mut code = [0xc0, 0x46].repeat(5);
        code.extend_from_slice(&[0x70, 0x47]);
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        memory.map(other.pc, 2);
        memory.write_range(other.pc, &[0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
        if queued_before_submission {
            let sampled_key = RegionKey { pc: entry.pc + 2, ..entry };
            jit.sample(sampled_key, Some((sampled_key, 8)));
        }
        jit.maintain(&memory);
        let request = responses.lock().requests[0].clone();
        for offset in [2, 4, 6, 8] {
            let sampled_key = RegionKey {
                pc: entry.pc + offset,
                ..entry
            };
            jit.sample(sampled_key, Some((sampled_key, 8)));
            jit.maintain(&memory);
        }
        jit.sample(other, Some((other, 8)));
        assert_eq!(responses.lock().requests.len(), 1);
        responses.lock().ready.push_back(completion(&request));
        for _ in 0..5 {
            jit.maintain(&memory);
        }
        let requests = &responses.lock().requests;
        assert_eq!(requests.len(), 2);
        assert_eq!(requests[1].regions.len(), 1);
        assert_eq!(
            requests[1].regions[0].ir.entry, other,
            "queued_before_submission={queued_before_submission}"
        );
    }
}

#[test]
fn queued_hot_regions_share_one_request_and_suppress_all_pending_aliases() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let entries: Vec<_> = (1..=5)
        .map(|page| RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        })
        .collect();
    for &entry in &entries {
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
    }
    jit.maintain(&memory);
    let request = responses.lock().requests[0].clone();
    assert_eq!(request.regions.iter().map(|region| region.ir.entry).collect::<Vec<_>>(), entries[..4]);
    assert!(request.regions.iter().all(|region| region.expected_old.is_none()));
    for &entry in &entries[..4] {
        for hits in 9..12 {
            let sampled_key = RegionKey { pc: entry.pc + 2, ..entry };
            jit.sample(sampled_key, Some((sampled_key, hits)));
            jit.maintain(&memory);
        }
        assert!(jit.lookup(entry, &memory).is_none());
    }
    jit.sample(entries[4], Some((entries[4], 9)));
    assert_eq!(responses.lock().requests.len(), 1);
    responses.lock().ready.push_back(completion(&request));
    jit.maintain(&memory);
    let next = responses.lock().requests[1].clone();
    assert_eq!(next.regions.len(), 1);
    assert_eq!(next.regions[0].ir.entry, entries[4]);
    for (slot, &entry) in entries[..4].iter().enumerate() {
        let handle = CompiledHandle {
            slot: slot as u32,
            generation: request.request,
        };
        assert_eq!(jit.lookup(entry, &memory), Some(handle));
        assert_eq!(jit.lookup(RegionKey { pc: entry.pc + 2, ..entry }, &memory), Some(handle));
    }
}

#[test]
fn invalid_failed_and_covered_candidates_do_not_block_later_queued_code() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let unmapped = RegionKey {
        pc: 0x10000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    let unsupported = RegionKey { pc: 0x20000, ..unmapped };
    let first = RegionKey { pc: 0x30000, ..unmapped };
    let later = RegionKey { pc: 0x40000, ..unmapped };
    memory.map(unsupported.pc, 2);
    memory.write_range(unsupported.pc, &[0x00, 0xdf]).unwrap();
    for entry in [first, later] {
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
    }
    for entry in [unmapped, unsupported, first, RegionKey { pc: first.pc + 2, ..first }] {
        jit.sample(entry, Some((entry, 8)));
    }
    jit.maintain(&memory);
    let request = responses.lock().requests[0].clone();
    assert_eq!(request.regions.len(), 1);
    assert_eq!(request.regions[0].ir.entry, first);
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    // A guest store does not publish a new source version, so the failed entry stays suppressed.
    memory.as_arm32cpu_memory().w16(unsupported.pc, 0x4770);
    for entry in [unsupported, first, later] {
        jit.sample(entry, Some((entry, 9)));
    }
    jit.maintain(&memory);
    let request = responses.lock().requests[1].clone();
    assert_eq!(request.regions.len(), 1);
    assert_eq!(request.regions[0].ir.entry, later);
}

#[test]
fn maximum_regions_fit_actual_serialized_admission_and_encoded_output_budgets() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut code = Vec::new();
    for _ in 0..64 {
        code.extend_from_slice(&[0xc0, 0x46].repeat(7));
        code.extend_from_slice(&0xd1f7u16.to_le_bytes()); // bne to this block's first instruction
    }
    code.extend_from_slice(&0xde00u16.to_le_bytes());
    let index_bytes = 512 * core::mem::size_of::<(RegionKey, u32)>();
    let mut retained = 0;
    let mut merged = false;
    for page in 1..=15 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        // A small final candidate leaves admission room to merge a maximum-sized old region.
        let source = if page == 15 { &[0xc0, 0x46, 0x00, 0xde][..] } else { &code };
        memory.map(entry.pc, source.len());
        memory.write_range(entry.pc, source).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(request.regions[0].ir.entry, entry);
        assert!(request.regions[0].expected_old.is_none());
        for region in &request.regions {
            let small = region.ir.entry == entry && page == 15;
            assert_eq!(region.ir.blocks.len(), if small { 1 } else { 64 });
            assert!(region.ir.blocks.iter().all(|block| block.instructions.len() == if small { 1 } else { 8 }));
            merged |= region.expected_old.is_some();
        }
        let payload = serde_json::to_vec(&request).unwrap();
        assert!(
            4 * responses.lock().request_ir_sizes.last().unwrap() + 5 * payload.len() + 2 * index_bytes * request.regions.len() + 2048 <= 1024 * 1024
        );
        retained += responses.lock().region_ir_sizes.last().unwrap()[0] + 3 * index_bytes;
        assert!(retained <= 1024 * 1024);
        let decoded = serde_json::from_slice(&payload).unwrap();
        let artifact = wie_arm_wasm::compile(&decoded).unwrap();
        assert_eq!(artifact.manifest.len(), request.regions.len());
        assert!(artifact.bytes.len() <= 512 * 1024);
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        for installed_page in 1..=page {
            assert!(
                jit.lookup(
                    RegionKey {
                        pc: installed_page * 0x10000 + if installed_page == 15 { 0 } else { 1022 },
                        ..entry
                    },
                    &memory
                )
                .is_some()
            );
        }
    }
    assert!(merged, "maximum regions must also exercise whole-generation merge admission");
}

#[test]
fn summed_new_reservations_leave_overflow_candidates_queued_before_merging_old_generations() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut old = Vec::new();
    for page in 1..=14 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let mut code = [0xc0, 0x46].repeat(256);
        code.extend_from_slice(&0xde00u16.to_le_bytes());
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        old.push((entry, jit.lookup(entry, &memory).unwrap()));
    }
    let entries: Vec<_> = (15..=18)
        .map(|page| RegionKey {
            pc: page * 0x10000,
            ..old[0].0
        })
        .collect();
    for &entry in &entries {
        let mut code = [0x00, 0x68].repeat(256); // ldr r0,[r0], with a full address operand in each IR instruction.
        code.extend_from_slice(&0xde00u16.to_le_bytes());
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
    }
    let mut submitted = 0;
    while submitted < entries.len() {
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        let new: Vec<_> = request.regions.iter().take_while(|region| region.expected_old.is_none()).collect();
        assert_eq!(new.len(), 2, "two large new regions fit, but the third must wait for the next request");
        assert_eq!(
            new.iter().map(|region| region.ir.entry).collect::<Vec<_>>(),
            entries[submitted..submitted + new.len()]
        );
        submitted += new.len();
        let raw_index_bytes: usize = request
            .regions
            .iter()
            .map(|region| region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>())
            .sum();
        assert!(4 * request.ir_size() + 5 * serde_json::to_vec(&request).unwrap().len() + 2 * raw_index_bytes <= 1024 * 1024);
        let attempts = responses.lock().requests.len();
        jit.maintain(&memory);
        assert_eq!(responses.lock().requests.len(), attempts);
        for &(entry, handle) in &old {
            assert_eq!(jit.lookup(entry, &memory), Some(handle));
        }
        assert!(
            request.regions.iter().all(|region| region.expected_old.is_none()),
            "new reservations leave no room for an old generation"
        );
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
    }
    assert_eq!(responses.lock().requests.len(), 16);
    for entry in entries {
        assert!(jit.lookup(entry, &memory).is_some(), "a deferred candidate must not be marked failed");
    }
}

#[test]
fn later_new_regions_can_subsume_earlier_new_and_installed_regions() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let original = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    let first = RegionKey { pc: 0x1004, ..original };
    let last = RegionKey { pc: 0x1006, ..original };
    // nop; bx lr; branch to original; branch to first.
    memory.map(original.pc, 8);
    memory
        .write_range(original.pc, &[0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7, 0xfd, 0xe7])
        .unwrap();
    jit.sample(original, Some((original, 8)));
    jit.maintain(&memory);
    let request = responses.lock().requests[0].clone();
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    let old = jit.lookup(original, &memory).unwrap();
    for entry in [first, last] {
        jit.sample(entry, Some((entry, 8)));
    }
    jit.maintain(&memory);
    let request = responses.lock().requests[1].clone();
    assert_eq!(request.regions.iter().map(|region| region.ir.entry).collect::<Vec<_>>(), [first, last]);
    let result = completion(&request);
    let redundant = result.result.as_ref().unwrap().regions[0].handle;
    let survivor = result.result.as_ref().unwrap().regions[1].handle;
    assert!(responses.lock().retired.is_empty());
    responses.lock().ready.push_back(result);
    jit.poll(&memory);
    assert_eq!(responses.lock().retired, [old, redundant]);
    for pc in [original.pc, original.pc + 2, first.pc, last.pc] {
        let key = RegionKey { pc, ..original };
        assert_eq!(jit.lookup(key, &memory), Some(survivor));
        jit.sample(key, Some((key, 9)));
    }
    jit.maintain(&memory);
    assert_eq!(responses.lock().requests.len(), 2);
}

#[test]
fn busy_batches_restore_all_candidates_in_order_even_without_a_live_generation() {
    for initially_installed in [false, true] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let entries: Vec<_> = (1..=8)
            .map(|page| RegionKey {
                pc: page * 0x10000,
                thumb: true,
                cpu_mode: 0x1f,
            })
            .collect();
        for entry in &entries {
            memory.map(entry.pc, 4);
            memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        }
        let mut generations = Vec::new();
        if initially_installed {
            for pair in entries[..4].chunks(2) {
                for &entry in pair {
                    jit.sample(entry, Some((entry, 8)));
                }
                jit.maintain(&memory);
                let request = responses.lock().requests.last().unwrap().clone();
                assert_eq!(request.regions.len(), 2);
                responses.lock().ready.push_back(completion(&request));
                jit.poll(&memory);
                generations.push(pair.iter().map(|&entry| jit.lookup(entry, &memory).unwrap()).collect::<Vec<_>>());
            }
        }
        for &entry in &entries[4..] {
            jit.sample(entry, Some((entry, 8)));
        }
        responses
            .lock()
            .admissions
            .extend([Admission::Busy, Admission::Busy, Admission::Accepted]);
        let mut generations = generations.into_iter();
        for attempt in 0..3 {
            let retired_before = responses.lock().retired.len();
            jit.maintain(&memory);
            let request = responses.lock().requests.last().unwrap().clone();
            assert_eq!(request.regions.iter().map(|region| region.ir.entry).collect::<Vec<_>>(), entries[4..]);
            if let Some(expected) = generations.next() {
                assert_eq!(responses.lock().retired[retired_before..], expected);
            } else {
                assert_eq!(responses.lock().retired.len(), retired_before);
            }
            for &entry in &entries[4..] {
                jit.sample(entry, Some((entry, 9)));
            }
            if attempt == 2 {
                responses.lock().ready.push_back(completion(&request));
                jit.poll(&memory);
                for &entry in &entries[4..] {
                    assert_eq!(jit.lookup(entry, &memory).unwrap().generation, request.request);
                }
            }
        }
        assert_eq!(responses.lock().requests.len(), 3 + 2 * usize::from(initially_installed));
    }
}

#[test]
fn pending_coverage_is_released_after_failure_or_stale_completion() {
    for outcome in ["failed", "stale", "admission_failed", "busy"] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let entry = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let interior = RegionKey { pc: entry.pc + 2, ..entry };
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        if outcome == "admission_failed" {
            responses.lock().admissions.push_back(Admission::Failed("submit failed".into()));
        } else if outcome == "busy" {
            responses.lock().admissions.push_back(Admission::Busy);
        }
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests[0].clone();
        if matches!(outcome, "failed" | "stale") {
            jit.sample(interior, Some((interior, 8)));
            jit.maintain(&memory);
            assert_eq!(responses.lock().requests.len(), 1);
            let mut result = completion(&request);
            if outcome == "failed" {
                result.result = Err("compile failed".into());
            } else {
                memory.write_range(entry.pc, &[0x01, 0x30]).unwrap();
            }
            responses.lock().ready.push_back(result);
            jit.poll(&memory);
        }
        assert!(jit.lookup(entry, &memory).is_none());
        assert!(jit.lookup(interior, &memory).is_none());
        jit.sample(interior, Some((interior, 9)));
        jit.maintain(&memory);
        let replacement = responses.lock().requests[1].clone();
        assert_eq!(
            replacement.regions[0].ir.entry,
            if outcome == "busy" { entry } else { interior },
            "{outcome}"
        );
        assert_eq!(replacement.regions[0].expected_old, None);
        responses.lock().ready.push_back(completion(&replacement));
        jit.poll(&memory);
        let current = jit.lookup(interior, &memory).unwrap();
        assert_eq!(current.generation, replacement.request);
        assert_eq!(jit.lookup(entry, &memory), (outcome == "busy").then_some(current));
    }
}

#[test]
fn multi_region_failures_are_atomic_and_release_all_pending_coverage() {
    for outcome in ["admission", "compile", "source", "manifest", "expected_old"] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let entries: Vec<_> = (1..=17)
            .map(|page| RegionKey {
                pc: page * 0x10000,
                thumb: true,
                cpu_mode: 0x1f,
            })
            .collect();
        let mut old = Vec::new();
        for (index, &entry) in entries.iter().enumerate() {
            memory.map(entry.pc, 4);
            memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
            if index < 14 {
                jit.sample(entry, Some((entry, 8)));
                jit.maintain(&memory);
                let request = responses.lock().requests.last().unwrap().clone();
                responses.lock().ready.push_back(completion(&request));
                jit.poll(&memory);
                old.push(jit.lookup(entry, &memory).unwrap());
            }
        }
        if outcome == "admission" {
            responses.lock().admissions.push_back(Admission::Failed("submit failed".into()));
        }
        for &entry in &entries[14..16] {
            jit.sample(entry, Some((entry, 8)));
        }
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(request.regions.len(), 4);
        assert_eq!(
            request.regions[..2].iter().map(|region| region.ir.entry).collect::<Vec<_>>(),
            entries[14..16]
        );
        let mut discarded = Vec::new();
        if outcome != "admission" {
            let mut result = completion(&request);
            if outcome == "compile" {
                result.result = Err("compile failed".into());
            } else {
                let artifact = result.result.as_mut().unwrap();
                discarded.extend(artifact.regions.iter().map(|region| region.handle));
                match outcome {
                    "source" => memory.write_range(entries[15].pc, &[0x01, 0x30]).unwrap(),
                    "manifest" => artifact.regions[1].manifest.entry.pc += 2,
                    "expected_old" => artifact.regions.last_mut().unwrap().manifest.expected_old.as_mut().unwrap().generation += 1,
                    _ => unreachable!(),
                }
            }
            responses.lock().ready.push_back(result);
            jit.poll(&memory);
        }
        assert_eq!(responses.lock().retired, discarded, "{outcome}");
        for (&entry, &handle) in entries[..14].iter().zip(&old) {
            assert_eq!(jit.lookup(entry, &memory), Some(handle), "{outcome}");
            assert_eq!(jit.lookup(RegionKey { pc: entry.pc + 2, ..entry }, &memory), Some(handle));
        }
        for &entry in &entries[14..16] {
            assert!(jit.lookup(entry, &memory).is_none(), "{outcome}");
            assert!(jit.lookup(RegionKey { pc: entry.pc + 2, ..entry }, &memory).is_none());
        }
        for &entry in &entries[14..] {
            jit.sample(entry, Some((entry, 9)));
        }
        jit.maintain(&memory);
        let retry = responses.lock().requests.last().unwrap().clone();
        assert_eq!(responses.lock().requests.len(), 16);
        let expected = if matches!(outcome, "admission" | "compile") {
            &entries[16..]
        } else {
            &entries[14..]
        };
        assert_eq!(
            retry
                .regions
                .iter()
                .take_while(|region| region.expected_old.is_none())
                .map(|region| region.ir.entry)
                .collect::<Vec<_>>(),
            expected,
            "every failed new source is suppressed, while rejected artifacts leave candidates retryable: {outcome}"
        );
    }
}

#[test]
fn each_new_regions_cost_is_preserved_for_future_whole_generation_merges() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let entries: Vec<_> = (1..=16)
        .map(|page| RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        })
        .collect();
    let mut old = Vec::new();
    for (index, &entry) in entries.iter().enumerate() {
        let count = match index {
            0 => 8,
            1 => 32,
            _ => 64,
        };
        let mut code = [0xc0, 0x46].repeat(count - 1);
        code.extend_from_slice(&[0x70, 0x47]);
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
        if index == 0 {
            continue;
        }
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        if index == 1 {
            assert_eq!(request.regions.len(), 2);
        } else if index < 15 {
            assert_eq!(request.regions.len(), 1);
        } else {
            // The first module costs 8+32 instructions, less than each 64-instruction module.
            // Charging both members the whole batch cost would incorrectly make it more expensive.
            assert_eq!(request.regions[0].ir.entry, entry);
            assert_eq!(
                request.regions.iter().filter_map(|region| region.expected_old).collect::<Vec<_>>(),
                old[..3]
            );
            assert_eq!(request.regions.len(), 4);
        }
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        if index == 1 {
            old.extend(entries[..2].iter().map(|&entry| jit.lookup(entry, &memory).unwrap()));
        } else {
            old.push(jit.lookup(entry, &memory).unwrap());
        }
    }
    for &entry in &entries {
        assert!(jit.lookup(entry, &memory).is_some());
    }
    assert_eq!(responses.lock().retired, old[..3]);
}

#[test]
fn summed_new_ir_reservations_evict_enough_cold_translations() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut live = BTreeMap::new();
    let mut next_page = 0;
    let mut code = [0xc0, 0x46].repeat(257);
    code.extend_from_slice(&0xde00u16.to_le_bytes());
    for page in 1..=128 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let before = live.len();
        live.retain(|key, _| jit.lookup(*key, &memory).is_some());
        let evicted = live.len() != before;
        let request = responses.lock().requests.last().unwrap().clone();
        let original_sizes = responses.lock().region_ir_sizes.last().unwrap().clone();
        assert!(
            original_sizes[0] > request.regions[0].ir_size(),
            "exercise capacity lost by a request clone"
        );
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        for (region, original_size) in request.regions.iter().zip(original_sizes) {
            let raw_index_bytes =
                region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>();
            live.insert(region.ir.entry, original_size + 2 * raw_index_bytes);
        }
        if evicted {
            next_page = page + 1;
            break;
        }
    }
    assert_ne!(next_page, 0, "fill the retained IR budget before submitting a batch");
    let entries: Vec<_> = (next_page..next_page + 4)
        .map(|page| RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        })
        .collect();
    for &entry in &entries {
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
    }
    jit.maintain(&memory);
    let request = responses.lock().requests.last().unwrap().clone();
    let original_sizes = responses.lock().region_ir_sizes.last().unwrap().clone();
    let new: Vec<_> = request.regions.iter().take_while(|region| region.expected_old.is_none()).collect();
    assert!(new.len() >= 2);
    let evicted: Vec<_> = live.keys().copied().filter(|key| jit.lookup(*key, &memory).is_none()).collect();
    assert!(evicted.len() >= new.len(), "eviction must reserve all new IR, not just the first region");
    assert_eq!(
        evicted,
        live.keys().copied().take(evicted.len()).collect::<Vec<_>>(),
        "evict the coldest old entries first"
    );
    for entry in evicted {
        live.remove(&entry);
    }
    let retained = live.values().sum::<usize>()
        + core::mem::size_of::<Box<[(RegionKey, u32)]>>()
        + new
            .iter()
            .zip(original_sizes)
            .map(|(region, original_size)| {
                let raw_index_bytes =
                    region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>();
                original_size + 2 * raw_index_bytes
            })
            .sum::<usize>();
    assert!(
        retained <= 1024 * 1024,
        "include every new region and both raw index reservations: {retained}"
    );
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    for region in &request.regions {
        assert!(jit.lookup(region.ir.entry, &memory).is_some());
    }
}

#[test]
fn fully_covered_translations_retire_only_after_validated_installation() {
    for thumb in [false, true] {
        for cpu_mode in [0x10, 0x1f] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
            let mut memory = EmulatedMemory::new();
            let first = RegionKey { pc: 0x1000, thumb, cpu_mode };
            let width = if thumb { 2 } else { 4 };
            let second = RegionKey {
                pc: first.pc + 2 * width,
                ..first
            };
            let code: Vec<_> = if thumb {
                [0x46c0u16, 0x4770, 0xe7fc].into_iter().flat_map(u16::to_le_bytes).collect()
            } else {
                [0xe1a00000u32, 0xe12fff1e, 0xeafffffc].into_iter().flat_map(u32::to_le_bytes).collect()
            };
            memory.map(first.pc, code.len());
            memory.write_range(first.pc, &code).unwrap();
            jit.sample(first, Some((first, 8)));
            jit.maintain(&memory);
            let original = responses.lock().requests[0].clone();
            responses.lock().ready.push_back(completion(&original));
            jit.poll(&memory);
            let old = jit.lookup(first, &memory).unwrap();

            jit.sample(second, Some((second, 8)));
            jit.maintain(&memory);
            let request = responses.lock().requests[1].clone();
            assert_eq!(jit.lookup(first, &memory), Some(old));
            assert!(responses.lock().retired.is_empty());
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            let current = jit.lookup(second, &memory).unwrap();
            assert_eq!(responses.lock().retired, [old]);
            for pc in [first.pc, first.pc + width, second.pc] {
                let key = RegionKey { pc, ..first };
                assert_eq!(jit.lookup(key, &memory), Some(current));
                jit.sample(key, Some((key, 99)));
                jit.maintain(&memory);
            }
            assert_eq!(responses.lock().requests.len(), 2);
            responses.lock().ready.push_back(completion(&original));
            jit.poll(&memory);
            assert_eq!(jit.lookup(first, &memory), Some(current));
            assert!(!responses.lock().retired.contains(&current));
        }
    }
}

#[test]
fn overlapping_regions_keep_canonical_ownership_and_surviving_aliases() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let first = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    let second = RegionKey { pc: 0x1006, ..first };
    let other = RegionKey { pc: 0x20000, ..first };
    // The second entry shares the tail, but does not cover the first instruction.
    memory.map(first.pc, 8);
    memory.write_range(first.pc, &[0xc0, 0x46, 0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7]).unwrap();
    memory.map(other.pc, 2);
    memory.write_range(other.pc, &[0x70, 0x47]).unwrap();
    let mut handles = Vec::new();
    for entry in [first, second] {
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        handles.push(jit.lookup(entry, &memory).unwrap());
    }
    assert_eq!(jit.lookup(first, &memory).unwrap(), handles[0]);
    assert_eq!(jit.lookup(second, &memory).unwrap(), handles[1]);
    responses.lock().admissions.push_back(Admission::Busy);
    jit.sample(other, Some((other, 8)));
    jit.maintain(&memory);
    assert_eq!(responses.lock().retired, [handles[0]]);
    assert!(jit.lookup(first, &memory).is_none());
    for pc in [first.pc + 2, first.pc + 4, second.pc] {
        assert_eq!(jit.lookup(RegionKey { pc, ..first }, &memory).unwrap(), handles[1]);
    }
    let sampled_key = RegionKey { pc: first.pc + 2, ..first };
    jit.sample(sampled_key, Some((sampled_key, 9)));
    jit.maintain(&memory);
    let request = responses.lock().requests.last().unwrap().clone();
    assert_eq!(request.regions[0].ir.entry, other);
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    jit.maintain(&memory);
    assert_eq!(responses.lock().requests.len(), 4);
    memory.write_range(first.pc, &[0x01, 0x30]).unwrap();
    assert!(jit.lookup(RegionKey { pc: first.pc + 2, ..first }, &memory).is_none());
    assert!(jit.lookup(first, &memory).is_none());
    assert!(jit.lookup(second, &memory).is_none());
    assert_eq!(responses.lock().retired, handles);
}

#[test]
fn covering_an_entry_without_its_entire_ir_preserves_the_old_owner() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let first = RegionKey {
        pc: 0x1002,
        thumb: true,
        cpu_mode: 0x1f,
    };
    let second = RegionKey { pc: 0x1000, ..first };
    let code = [0xc0, 0x46].repeat(513);
    memory.map(second.pc, code.len());
    memory.write_range(second.pc, &code).unwrap();
    let mut handles = Vec::new();
    for entry in [first, second] {
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(request.regions[0].ir.blocks[0].instructions.len(), 512);
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        handles.push(jit.lookup(entry, &memory).unwrap());
    }
    assert_eq!(jit.lookup(first, &memory), Some(handles[0]));
    assert_eq!(jit.lookup(RegionKey { pc: 0x1400, ..first }, &memory), Some(handles[0]));
    assert_eq!(jit.lookup(second, &memory), Some(handles[1]));
    assert!(responses.lock().retired.is_empty());
}

#[test]
fn covered_pcs_preserve_distinct_instructions_and_cpu_modes() {
    for (cpu_mode, mutate) in [(0x10, false), (0x1f, true)] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let first = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let second = RegionKey {
            pc: 0x1004,
            cpu_mode,
            ..first
        };
        memory.map(first.pc, 6);
        memory.write_range(first.pc, &[0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7]).unwrap();
        jit.sample(first, Some((first, 8)));
        jit.maintain(&memory);
        let original = responses.lock().requests[0].clone();
        responses.lock().ready.push_back(completion(&original));
        jit.poll(&memory);
        let old = jit.lookup(first, &memory).unwrap();
        if mutate {
            memory.as_arm32cpu_memory().w16(first.pc, 0x3001);
        }
        jit.sample(second, Some((second, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests[1].clone();
        assert_eq!(original.regions[0].source, request.regions[0].source);
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        assert_eq!(jit.lookup(first, &memory), Some(old));
        assert_ne!(jit.lookup(second, &memory), Some(old));
        assert!(responses.lock().retired.is_empty());
    }
}

#[test]
fn failed_or_stale_covering_completions_do_not_prune_installed_owners() {
    for outcome in ["failed", "manifest", "source"] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let first = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let second = RegionKey { pc: 0x1004, ..first };
        memory.map(first.pc, 6);
        memory.write_range(first.pc, &[0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7]).unwrap();
        jit.sample(first, Some((first, 8)));
        jit.maintain(&memory);
        let original = responses.lock().requests[0].clone();
        responses.lock().ready.push_back(completion(&original));
        jit.poll(&memory);
        let old = jit.lookup(first, &memory).unwrap();
        jit.sample(second, Some((second, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests[1].clone();
        let mut result = completion(&request);
        let discarded = result.result.as_ref().unwrap().regions[0].handle;
        if outcome == "failed" {
            result.result = Err("compile failed".into());
        } else if outcome == "manifest" {
            result.result.as_mut().unwrap().regions[0].manifest.expected_old = Some(old);
        } else {
            memory.write_range(first.pc, &[0x01, 0x30]).unwrap();
        }
        responses.lock().ready.push_back(result);
        jit.poll(&memory);
        assert_eq!(responses.lock().retired, if outcome == "failed" { vec![] } else { vec![discarded] });
        assert_eq!(jit.lookup(first, &memory), (outcome != "source").then_some(old));
        assert!(jit.lookup(second, &memory).is_none());
    }
}

#[test]
fn subsumption_retires_recompiled_fragments_without_dropping_other_exports() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let first = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    let covering = RegionKey { pc: 0x1004, ..first };
    for pc in core::iter::once(first.pc).chain((2..=14).map(|page| page * 0x10000)) {
        memory.map(pc, 6);
        memory.write_range(pc, &[0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7]).unwrap();
        let sampled_key = RegionKey { pc, ..first };
        jit.sample(sampled_key, Some((sampled_key, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
    }
    let old = jit.lookup(first, &memory).unwrap();
    jit.sample(covering, Some((covering, 8)));
    jit.maintain(&memory);
    let request = responses.lock().requests.last().unwrap().clone();
    assert_eq!(request.regions.len(), 3);
    assert_eq!(
        request.regions.iter().find(|region| region.ir.entry == first).unwrap().expected_old,
        Some(old)
    );
    let result = completion(&request);
    let compiled = &result.result.as_ref().unwrap().regions;
    let redundant = compiled.iter().find(|region| region.manifest.entry == first).unwrap().handle;
    let survivor = compiled.iter().find(|region| region.manifest.entry == covering).unwrap().handle;
    let other = compiled
        .iter()
        .find(|region| region.manifest.entry != first && region.manifest.entry != covering)
        .unwrap();
    let other_entry = other.manifest.entry;
    let other_handle = other.handle;
    let mut expected_retired: BTreeSet<_> = request.regions.iter().filter_map(|region| region.expected_old).collect();
    expected_retired.insert(redundant);
    assert!(responses.lock().retired.is_empty());
    responses.lock().ready.push_back(result);
    jit.poll(&memory);
    assert_eq!(responses.lock().retired.len(), expected_retired.len());
    assert_eq!(responses.lock().retired.iter().copied().collect::<BTreeSet<_>>(), expected_retired);
    assert_eq!(jit.lookup(first, &memory), Some(survivor));
    assert_eq!(jit.lookup(RegionKey { pc: first.pc + 2, ..first }, &memory), Some(survivor));
    assert_eq!(jit.lookup(other_entry, &memory), Some(other_handle));
    for page in 15..=18 {
        let entry = RegionKey { pc: page * 0x10000, ..first };
        memory.map(entry.pc, 2);
        memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert!(request.regions.iter().all(|region| region.ir.entry != first));
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        assert_eq!(jit.lookup(first, &memory).unwrap(), jit.lookup(covering, &memory).unwrap());
    }
}

#[test]
fn duplicate_completion_does_not_retire_an_installed_function() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    memory.map(0x1000, 2);
    memory.write_range(0x1000, &[0x70, 0x47]).unwrap();
    let entry = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    jit.sample(entry, Some((entry, 8)));
    jit.maintain(&memory);
    let request = responses.lock().requests[0].clone();
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    let installed = jit.lookup(entry, &memory).unwrap();
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    assert_eq!(jit.lookup(entry, &memory).unwrap(), installed);
    assert!(!responses.lock().retired.contains(&installed));
}

#[test]
fn foreign_completions_do_not_consume_the_pending_request() {
    for (other_session, failed) in [(true, false), (false, false), (true, true), (false, true)] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        memory.map(0x1000, 2);
        memory.write_range(0x1000, &[0x70, 0x47]).unwrap();
        let entry = RegionKey {
            pc: 0x1000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests[0].clone();
        let discarded = CompiledHandle { slot: 0, generation: 99 };
        let mut foreign = completion(&request);
        if other_session {
            foreign.session += 1;
        } else {
            foreign.request += 1;
        }
        if failed {
            foreign.result = Err("unrelated failure".into());
        } else {
            foreign.result.as_mut().unwrap().regions[0].handle = discarded;
        }
        responses.lock().ready.push_back(foreign);
        jit.poll(&memory);
        assert!(jit.lookup(entry, &memory).is_none());
        assert_eq!(responses.lock().retired, if failed { vec![] } else { vec![discarded] });
        for hits in 9..20 {
            jit.sample(entry, Some((entry, hits)));
            jit.maintain(&memory);
        }
        assert_eq!(responses.lock().requests.len(), 1);
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        let installed = jit.lookup(entry, &memory).unwrap();
        assert_eq!(
            installed,
            CompiledHandle {
                slot: 0,
                generation: request.request
            }
        );
        assert!(!responses.lock().retired.contains(&installed));
    }
}

#[test]
fn stale_source_or_old_handle_discards_the_whole_merge() {
    for invalidation in ["source", "retired", "expected_old"] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let mut installed = Vec::new();
        for page in 1..=14 {
            let entry = RegionKey {
                pc: page * 0x10000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            memory.map(entry.pc, 4);
            memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
            jit.sample(entry, Some((entry, 8)));
            jit.maintain(&memory);
            let request = responses.lock().requests.last().unwrap().clone();
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            installed.push((entry, jit.lookup(entry, &memory).unwrap()));
        }
        let entry = RegionKey {
            pc: 0xf0000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 2);
        memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(request.regions.len(), 3);
        let (old_entry, old_handle) = installed[0];
        assert_eq!(
            request.regions.iter().find(|region| region.ir.entry == old_entry).unwrap().expected_old,
            Some(old_handle)
        );
        let mut result = completion(&request);
        let artifact = result.result.as_mut().unwrap();
        let discarded: Vec<_> = artifact.regions.iter().map(|region| region.handle).collect();
        if invalidation == "expected_old" {
            artifact
                .regions
                .iter_mut()
                .find(|region| region.manifest.entry == old_entry)
                .unwrap()
                .manifest
                .expected_old = Some(CompiledHandle {
                generation: old_handle.generation + 1,
                ..old_handle
            });
            assert!(request.regions.iter().all(|region| memory.code_is_current(&region.source)));
        } else {
            memory.write_range(old_entry.pc, &[0xc0, 0x46]).unwrap();
            if invalidation == "retired" {
                assert!(
                    jit.lookup(
                        RegionKey {
                            pc: old_entry.pc + 2,
                            ..old_entry
                        },
                        &memory
                    )
                    .is_none()
                );
                assert!(jit.lookup(old_entry, &memory).is_none());
                assert_eq!(responses.lock().retired, [old_handle]);
            }
        }
        responses.lock().ready.push_back(result);
        jit.poll(&memory);
        assert!(jit.lookup(entry, &memory).is_none());
        for (entry, handle) in &installed[usize::from(invalidation != "expected_old")..] {
            assert_eq!(jit.lookup(*entry, &memory).unwrap(), *handle);
            assert_eq!(jit.lookup(RegionKey { pc: entry.pc + 2, ..*entry }, &memory).unwrap(), *handle);
            assert!(!responses.lock().retired.contains(handle));
        }
        assert!(responses.lock().retired.ends_with(&discarded));
    }
}

#[test]
fn merges_replace_exact_old_handles_and_late_generations_stay_retired() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut entries = Vec::new();
    let mut merges = 0;
    for page in 1..=21 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        if page > 15 {
            let before: Vec<_> = entries.iter().map(|key| jit.lookup(*key, &memory).unwrap()).collect();
            let old_request = responses.lock().requests[0].clone();
            responses.lock().ready.push_back(completion(&old_request));
            let retired_before = responses.lock().retired.len();
            jit.poll(&memory);
            for (key, handle) in entries.iter().zip(before) {
                assert_eq!(jit.lookup(*key, &memory).unwrap(), handle);
                assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..*key }, &memory).unwrap(), handle);
                assert!(!responses.lock().retired[retired_before..].contains(&handle));
            }
            assert!(jit.lookup(entry, &memory).is_none());
        }
        let old_handles: Vec<_> = request.regions.iter().filter_map(|region| region.expected_old).collect();
        if page <= 14 {
            assert!(old_handles.is_empty(), "do not recompile below the module merge threshold");
        }
        for region in &request.regions {
            assert_eq!(jit.lookup(region.ir.entry, &memory), region.expected_old);
        }
        merges += usize::from(!old_handles.is_empty());
        let result = completion(&request);
        let replacements: Vec<_> = result
            .result
            .as_ref()
            .unwrap()
            .regions
            .iter()
            .map(|region| (region.manifest.entry, region.handle))
            .collect();
        let retired_before = responses.lock().retired.len();
        responses.lock().ready.push_back(result);
        jit.poll(&memory);
        assert_eq!(responses.lock().retired[retired_before..], old_handles);
        for (key, handle) in replacements {
            assert_eq!(jit.lookup(key, &memory).unwrap(), handle);
            assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory).unwrap(), handle);
            assert!(!old_handles.contains(&handle));
        }
        entries.push(entry);
    }
    assert_eq!(merges, 4);
}

#[test]
fn busy_admission_reclaims_cold_handles_and_retries_at_maintenance_boundaries() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut installed = Vec::new();
    for page in 1..=2 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        installed.push((entry, jit.lookup(entry, &memory).unwrap()));
    }
    let entry = RegionKey {
        pc: 0x30000,
        ..installed[0].0
    };
    memory.map(entry.pc, 2);
    memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
    let sampled_key = RegionKey {
        pc: installed[0].0.pc + 2,
        ..installed[0].0
    };
    jit.sample(sampled_key, Some((sampled_key, 9)));
    jit.sample(entry, Some((entry, 8)));
    responses
        .lock()
        .admissions
        .extend([Admission::Busy, Admission::Busy, Admission::Accepted]);
    for (attempt, (cold, handle)) in installed.iter().rev().enumerate() {
        jit.maintain(&memory);
        assert_eq!(responses.lock().requests.len(), 3 + attempt);
        assert_eq!(responses.lock().retired.last(), Some(handle));
        assert!(jit.lookup(*cold, &memory).is_none());
        assert!(jit.lookup(RegionKey { pc: cold.pc + 2, ..*cold }, &memory).is_none());
        assert!(jit.lookup(entry, &memory).is_none());
        for hits in 9..20 {
            jit.sample(entry, Some((entry, hits)));
            jit.poll(&memory);
        }
        assert_eq!(responses.lock().requests.len(), 3 + attempt);
    }
    jit.maintain(&memory);
    let request = responses.lock().requests.last().unwrap().clone();
    assert_eq!(responses.lock().requests.len(), 5);
    assert_eq!(request.regions[0].source, responses.lock().requests[2].regions[0].source);
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    let handle = jit.lookup(entry, &memory).unwrap();
    assert_eq!(
        handle,
        CompiledHandle {
            slot: 0,
            generation: request.request
        }
    );
    assert!(!responses.lock().retired.contains(&handle));
}

#[test]
fn whole_generation_merges_retain_a_stable_working_set() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut entries = Vec::new();
    let mut largest_module = 0;
    let mut most_replaced_generations = 0;
    for page in 1..=256 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        let before: Vec<_> = entries.iter().map(|key| (*key, jit.lookup(*key, &memory).unwrap())).collect();
        let retired_before = responses.lock().retired.len();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        assert_eq!(responses.lock().retired.len(), retired_before, "the small working set needs no eviction");
        assert_eq!(responses.lock().requests.len(), page as usize);
        let request = responses.lock().requests.last().unwrap().clone();
        let old_handles: Vec<_> = request.regions.iter().filter_map(|region| region.expected_old).collect();
        let replaced_generations: BTreeSet<_> = old_handles.iter().map(|handle| handle.generation).collect();
        most_replaced_generations = most_replaced_generations.max(replaced_generations.len());
        for generation in replaced_generations {
            let installed: BTreeSet<_> = before
                .iter()
                .map(|(_, handle)| *handle)
                .filter(|handle| handle.generation == generation)
                .collect();
            let replaced: BTreeSet<_> = old_handles.iter().copied().filter(|handle| handle.generation == generation).collect();
            assert_eq!(
                replaced, installed,
                "request {} partially replaces generation {generation}",
                request.request
            );
        }
        let index_bytes: usize = request
            .regions
            .iter()
            .map(|region| region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>())
            .sum();
        assert!(4 * request.ir_size() + 5 * serde_json::to_vec(&request).unwrap().len() + 2 * index_bytes <= 1024 * 1024);
        largest_module = largest_module.max(request.regions.len());
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        assert_eq!(responses.lock().retired[retired_before..], old_handles);
        for (key, handle) in before {
            let current = jit.lookup(key, &memory).unwrap();
            assert_eq!(current != handle, old_handles.contains(&handle));
            assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory).unwrap(), current);
        }
        entries.push(entry);
    }
    assert!(largest_module > 16, "whole generations must grow beyond the old fifteen-replacement cap");
    assert_eq!(most_replaced_generations, 2);
    let retired_before = responses.lock().retired.len();
    for _ in 0..3 {
        for &entry in &entries {
            jit.sample(entry, Some((entry, 99)));
            jit.maintain(&memory);
        }
    }
    assert_eq!(
        responses.lock().requests.len(),
        entries.len(),
        "stable code must stop compiling after warmup"
    );
    assert_eq!(responses.lock().retired.len(), retired_before);
}

#[test]
fn busy_admission_reclaims_whole_generations_by_their_most_recent_sample() {
    for (keep_merged_generation, known_entry) in [(false, false), (false, true), (true, false), (true, true)] {
        let responses = Arc::new(Mutex::new(Responses::default()));
        let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
        let mut memory = EmulatedMemory::new();
        let mut entries = Vec::new();
        for page in 1..=15 {
            let entry = RegionKey {
                pc: page * 0x10000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            memory.map(entry.pc, 4);
            memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
            jit.sample(entry, Some((entry, 8)));
            jit.maintain(&memory);
            let request = responses.lock().requests.last().unwrap().clone();
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            entries.push(entry);
        }
        let before: Vec<_> = entries.iter().map(|key| (*key, jit.lookup(*key, &memory).unwrap())).collect();
        let mut generations: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for &(entry, handle) in &before {
            generations.entry(handle.generation).or_default().push((entry, handle));
        }
        let (&merged, members) = generations.iter().find(|(_, members)| members.len() >= 3).unwrap();
        let cold = if keep_merged_generation {
            // One recent member protects a module even when its other members are older than the victim.
            let entry = members.last().unwrap().0;
            jit.sample(
                RegionKey { pc: entry.pc + 2, ..entry },
                known_entry.then_some((RegionKey { pc: 0x90000, ..entry }, 1)),
            );
            *generations.keys().find(|&&generation| generation != merged).unwrap()
        } else {
            for (&generation, members) in &generations {
                if generation != merged {
                    let entry = members[0].0;
                    jit.sample(
                        RegionKey { pc: entry.pc + 2, ..entry },
                        known_entry.then_some((RegionKey { pc: 0x90000, ..entry }, 1)),
                    );
                }
            }
            merged
        };
        let entry = RegionKey { pc: 0x100000, ..entries[0] };
        memory.map(entry.pc, 2);
        memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
        responses.lock().admissions.push_back(Admission::Busy);
        let retired_before = responses.lock().retired.len();
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let expected: BTreeSet<_> = generations[&cold].iter().map(|(_, handle)| *handle).collect();
        assert_eq!(
            responses.lock().retired[retired_before..].iter().copied().collect::<BTreeSet<_>>(),
            expected
        );
        for (key, handle) in before {
            let expected = (handle.generation != cold).then_some(handle);
            assert_eq!(jit.lookup(key, &memory), expected);
            assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory), expected);
        }
        let attempts = responses.lock().requests.len();
        jit.sample(entry, Some((entry, 9)));
        jit.poll(&memory);
        assert_eq!(responses.lock().requests.len(), attempts);
        jit.maintain(&memory);
        assert_eq!(responses.lock().requests.len(), attempts + 1);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        assert_eq!(jit.lookup(entry, &memory).unwrap().generation, request.request);
    }
}

#[test]
fn ir_pressure_bounds_merge_requests_and_reclaims_cold_translations() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let mut entries = Vec::new();
    let mut evictions = 0;
    for page in 1..=128 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        let mut code = [0xc0, 0x46].repeat(256);
        code.extend_from_slice(&0xde00u16.to_le_bytes());
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, Some((entry, 8)));
        let before: Vec<_> = entries
            .iter()
            .filter_map(|(key, _)| jit.lookup(*key, &memory).map(|value| (*key, value)))
            .collect();
        let retired_before = responses.lock().retired.len();
        jit.maintain(&memory);
        assert_eq!(responses.lock().requests.len(), page as usize);
        let request = responses.lock().requests.last().unwrap().clone();
        let index_bytes: usize = request
            .regions
            .iter()
            .map(|region| region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>())
            .sum();
        assert!(
            4 * responses.lock().request_ir_sizes.last().unwrap() + 5 * serde_json::to_vec(&request).unwrap().len() + 2 * index_bytes <= 1024 * 1024
        );
        if page == 15 {
            assert!(
                (2..15).contains(&request.regions.len()),
                "IR budget must limit the otherwise eligible fourteen old regions"
            );
        }
        for (key, handle) in before {
            if jit.lookup(key, &memory).is_none() {
                assert!(responses.lock().retired[retired_before..].contains(&handle));
                assert!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory).is_none());
                evictions += 1;
            }
        }
        entries.push((entry, 0));
        for (region, original_size) in request.regions.iter().zip(responses.lock().region_ir_sizes.last().unwrap()) {
            let instruction_count = region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>();
            entries.iter_mut().find(|(key, _)| *key == region.ir.entry).unwrap().1 =
                original_size + 2 * instruction_count * core::mem::size_of::<(RegionKey, u32)>();
        }
        let old_handles: Vec<_> = entries.iter().filter_map(|(key, _)| jit.lookup(*key, &memory)).collect();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        let handle = jit.lookup(entry, &memory).unwrap();
        assert!(!old_handles.contains(&handle));
        assert!(!responses.lock().retired.contains(&handle));
        let live_sizes: Vec<_> = entries
            .iter()
            .filter_map(|(key, size)| jit.lookup(*key, &memory).map(|_| *size))
            .collect();
        assert!(live_sizes.iter().sum::<usize>() + core::mem::size_of::<Box<[(RegionKey, u32)]>>() <= 1024 * 1024);
        if evictions == 2 {
            break;
        }
    }
    assert_eq!(evictions, 2, "must cross the retained IR budget and continue installing");
    assert!(entries.iter().map(|(_, size)| size).sum::<usize>() > 1024 * 1024);
}

#[test]
fn overlapping_ir_pressure_preserves_canonical_owners_and_surviving_aliases() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    let code = [0xc0, 0x46].repeat(639);
    memory.map(0x11000, code.len());
    memory.write_range(0x11000, &code).unwrap();
    let mut installed = BTreeMap::new();
    let mut evictions = 0;
    for offset in 0..128 {
        // Each earlier window loses one instruction from the previous tail, preventing subsumption.
        let entry = RegionKey {
            pc: 0x11000 + (127 - offset) * 2,
            thumb: true,
            cpu_mode: 0x1f,
        };
        jit.sample(entry, Some((entry, 8)));
        jit.maintain(&memory);
        let retired = responses.lock().retired.clone();
        let evicted: Vec<_> = installed
            .iter()
            .filter(|(_, (handle, _))| retired.contains(handle))
            .map(|(key, _)| *key)
            .collect();
        evictions += evicted.len();
        installed.retain(|_, (handle, _)| !retired.contains(handle));
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(responses.lock().requests.len(), offset as usize + 1);
        assert_eq!(request.regions[0].ir.entry, entry);
        let raw_index_bytes: usize = request
            .regions
            .iter()
            .map(|region| region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * core::mem::size_of::<(RegionKey, u32)>())
            .sum();
        assert!(
            4 * responses.lock().request_ir_sizes.last().unwrap() + 5 * serde_json::to_vec(&request).unwrap().len() + 2 * raw_index_bytes
                <= 1024 * 1024
        );
        let result = completion(&request);
        for ((region, compiled), original_size) in request
            .regions
            .iter()
            .zip(&result.result.as_ref().unwrap().regions)
            .zip(responses.lock().region_ir_sizes.last().unwrap())
        {
            let instruction_count = region.ir.blocks[0].instructions.len();
            assert_eq!(instruction_count, 512);
            installed.insert(
                region.ir.entry,
                (
                    compiled.handle,
                    original_size + 2 * instruction_count * core::mem::size_of::<(RegionKey, u32)>(),
                ),
            );
        }
        responses.lock().ready.push_back(result);
        jit.poll(&memory);
        let retained = installed.values().map(|(_, size)| size).sum::<usize>() + core::mem::size_of::<Box<[(RegionKey, u32)]>>();
        assert!(retained <= 1024 * 1024, "raw index rebuild reservation exceeded: {retained}");
        for (key, (handle, _)) in &installed {
            assert_eq!(jit.lookup(*key, &memory), Some(*handle));
        }
        for key in evicted {
            let survivor = jit.lookup(key, &memory).unwrap();
            assert!(installed.values().any(|(handle, _)| *handle == survivor));
        }
        if evictions >= 2 {
            break;
        }
    }
    assert!(evictions >= 2, "overlapping regions must cross the retained IR budget");
}

#[test]
fn compilation_failure_is_suppressed_only_for_the_current_code_version() {
    let responses = Arc::new(Mutex::new(Responses::default()));
    let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
    let mut memory = EmulatedMemory::new();
    memory.map(0x1000, 2);
    memory.write_range(0x1000, &[0x70, 0x47]).unwrap();
    let entry = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x1f,
    };
    jit.sample(entry, Some((entry, 8)));
    jit.maintain(&memory);
    responses.lock().ready.push_back(CompileCompletion {
        session: 1,
        request: 1,
        result: Err("compile failed".into()),
    });
    jit.poll(&memory);
    for hits in 9..20 {
        jit.sample(entry, Some((entry, hits)));
        jit.maintain(&memory);
    }
    assert_eq!(responses.lock().requests.len(), 1);
    memory.write_range(0x1000, &[0x08, 0x47]).unwrap();
    jit.sample(entry, Some((entry, 20)));
    jit.maintain(&memory);
    assert_eq!(responses.lock().requests.len(), 2);
}

#[test]
fn explicit_cache_publication_expires_installed_pending_and_failed_translations() {
    for opcode in [0xee070f15u32, 0xee070f35] {
        for state in ["installed", "pending", "failed"] {
            let responses = Arc::new(Mutex::new(Responses::default()));
            let mut jit = Jit::new(1, Box::new(DeferredExecutor(responses.clone())));
            let mut engine = Arm32CpuEngine::new();
            let entry = RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            engine.mem_map(entry.pc, 2, MemoryPermission::ReadWriteExecute);
            engine.mem_write(entry.pc, &0x4700u16.to_le_bytes()).unwrap(); // bx r0
            engine.mem_map(0x20000, 4, MemoryPermission::ReadWriteExecute);
            engine.mem_write(0x20000, &opcode.to_le_bytes()).unwrap();
            jit.sample(entry, Some((entry, 8)));
            jit.maintain(&engine.mem);
            let request = responses.lock().requests[0].clone();
            let complete = completion(&request);
            let old = complete.result.as_ref().unwrap().regions[0].handle;
            if state != "pending" {
                let mut result = completion(&request);
                if state == "failed" {
                    result.result = Err("compile failed".into());
                }
                responses.lock().ready.push_back(result);
                jit.poll(&engine.mem);
            }

            engine.mem.as_arm32cpu_memory().w16(entry.pc, 0x4770); // bx lr
            assert!(engine.mem.code_is_current(&request.regions[0].source));
            if state == "installed" {
                assert_eq!(jit.lookup(entry, &engine.mem), Some(old));
            }
            for hits in 9..12 {
                jit.sample(entry, Some((entry, hits)));
                jit.maintain(&engine.mem);
            }
            assert_eq!(responses.lock().requests.len(), 1, "{state}");

            engine.reg_write(ArmRegister::Cpsr, 0x1f);
            engine.reg_write(ArmRegister::PC, 0x20000);
            engine.reg_write(ArmRegister::R0, entry.pc);
            let result = engine.run(0x20004, 1, None).unwrap();
            assert_eq!(result.instructions_executed, 1);
            assert!(!engine.mem.code_is_current(&request.regions[0].source));
            if state == "pending" {
                responses.lock().ready.push_back(complete);
                jit.poll(&engine.mem);
            }
            assert!(jit.lookup(entry, &engine.mem).is_none(), "{state}");
            assert_eq!(responses.lock().retired, if state == "failed" { vec![] } else { vec![old] });
            jit.sample(entry, Some((entry, 12)));
            jit.maintain(&engine.mem);
            let replacement = responses.lock().requests[1].clone();
            assert!(matches!(
                replacement.regions[0].ir.blocks[0].instructions[0].operation,
                wie_arm_jit::Operation::Branch {
                    target: wie_arm_jit::Value::Register(14),
                    ..
                }
            ));
            responses.lock().ready.push_back(completion(&replacement));
            jit.poll(&engine.mem);
            assert_ne!(jit.lookup(entry, &engine.mem), Some(old));
            assert!(jit.lookup(entry, &engine.mem).is_some());
        }
    }
}
