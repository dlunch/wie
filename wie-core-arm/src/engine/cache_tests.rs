use alloc::{collections::VecDeque, string::String, sync::Arc, vec};

use spin::Mutex;
use wie_arm_jit::{
    Admission, CompileCompletion, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion, Operation,
};

use super::*;

#[derive(Default)]
struct Responses {
    requests: Vec<CompileRequest>,
    admissions: VecDeque<Admission>,
    ready: VecDeque<CompileCompletion>,
    retired: Vec<CompiledHandle>,
}

struct DeferredExecutor(Arc<Mutex<Responses>>);

impl CompiledExecutor for DeferredExecutor {
    fn submit(&mut self, request: CompileRequest) -> Admission {
        let mut responses = self.0.lock();
        responses.requests.push(request);
        responses.admissions.pop_front().unwrap_or(Admission::Accepted)
    }

    fn poll(&mut self) -> Option<CompileCompletion> {
        self.0.lock().ready.pop_front()
    }

    fn execute(&mut self, _: CompiledHandle, _: &mut RunFrame, _: &mut dyn ExecutionAccess) -> core::result::Result<CompiledExit, String> {
        panic!("cache tests do not execute a CPU");
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
            jit.sample(entry, 8);
            jit.maintain(&memory);
            let request = responses.lock().requests[0].clone();
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            let installed = jit.lookup(entry, &memory).unwrap();
            for offset in [width, 4 * width] {
                let interior = RegionKey {
                    pc: entry.pc + offset,
                    ..entry
                };
                assert_eq!(jit.lookup(interior, &memory), Some(installed), "thumb={thumb}, mode={cpu_mode:#x}");
                jit.sample(interior, 8);
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
            assert_eq!(responses.lock().retired, [installed.0]);
            jit.sample(interior, 9);
            jit.maintain(&memory);
            let replacement = responses.lock().requests[1].clone();
            assert_eq!(replacement.regions[0].ir.entry, interior);
            assert_eq!(replacement.regions[0].expected_old, None);
            responses.lock().ready.push_back(completion(&replacement));
            jit.poll(&memory);
            let current = jit.lookup(interior, &memory).unwrap().0;
            assert_ne!(current, installed.0);
            assert_eq!(
                jit.lookup(
                    RegionKey {
                        pc: entry.pc + 4 * width,
                        ..entry
                    },
                    &memory
                )
                .unwrap()
                .0,
                current
            );
            assert!(jit.lookup(entry, &memory).is_none());
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            assert_eq!(jit.lookup(interior, &memory).unwrap().0, current);
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
        jit.sample(entry, 8);
        if queued_before_submission {
            jit.sample(RegionKey { pc: entry.pc + 2, ..entry }, 8);
        }
        jit.maintain(&memory);
        let request = responses.lock().requests[0].clone();
        for offset in [2, 4, 6, 8] {
            jit.sample(
                RegionKey {
                    pc: entry.pc + offset,
                    ..entry
                },
                8,
            );
            jit.maintain(&memory);
        }
        jit.sample(other, 8);
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
        jit.sample(entry, 8);
        jit.maintain(&memory);
        let request = responses.lock().requests[0].clone();
        if matches!(outcome, "failed" | "stale") {
            jit.sample(interior, 8);
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
        jit.sample(interior, 9);
        jit.maintain(&memory);
        let replacement = responses.lock().requests[1].clone();
        assert_eq!(replacement.regions[0].ir.entry, interior, "{outcome}");
        assert_eq!(replacement.regions[0].expected_old, None);
        responses.lock().ready.push_back(completion(&replacement));
        jit.poll(&memory);
        assert_eq!(jit.lookup(interior, &memory).unwrap().0.generation, replacement.request);
        assert!(jit.lookup(entry, &memory).is_none());
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
    let second = RegionKey { pc: 0x1004, ..first };
    let other = RegionKey { pc: 0x20000, ..first };
    // The second entry reaches the first translation through a backward branch.
    memory.map(first.pc, 6);
    memory.write_range(first.pc, &[0xc0, 0x46, 0x70, 0x47, 0xfc, 0xe7]).unwrap();
    memory.map(other.pc, 2);
    memory.write_range(other.pc, &[0x70, 0x47]).unwrap();
    let mut handles = Vec::new();
    for entry in [first, second] {
        jit.sample(entry, 8);
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        handles.push(jit.lookup(entry, &memory).unwrap().0);
    }
    assert_eq!(jit.lookup(first, &memory).unwrap().0, handles[0]);
    assert_eq!(jit.lookup(second, &memory).unwrap().0, handles[1]);
    responses.lock().admissions.push_back(Admission::Busy);
    jit.sample(other, 8);
    jit.maintain(&memory);
    assert_eq!(responses.lock().retired, [handles[0]]);
    for pc in [first.pc, first.pc + 2, second.pc] {
        assert_eq!(jit.lookup(RegionKey { pc, ..first }, &memory).unwrap().0, handles[1]);
    }
    jit.sample(first, 9);
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
    jit.sample(entry, 8);
    jit.maintain(&memory);
    let request = responses.lock().requests[0].clone();
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    let installed = jit.lookup(entry, &memory).unwrap().0;
    responses.lock().ready.push_back(completion(&request));
    jit.poll(&memory);
    assert_eq!(jit.lookup(entry, &memory).unwrap().0, installed);
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
        jit.sample(entry, 8);
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
            jit.sample(entry, hits);
            jit.maintain(&memory);
        }
        assert_eq!(responses.lock().requests.len(), 1);
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        let installed = jit.lookup(entry, &memory).unwrap().0;
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
        for page in 1..=6 {
            let entry = RegionKey {
                pc: page * 0x10000,
                thumb: true,
                cpu_mode: 0x1f,
            };
            memory.map(entry.pc, 4);
            memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
            jit.sample(entry, 8);
            jit.maintain(&memory);
            let request = responses.lock().requests.last().unwrap().clone();
            responses.lock().ready.push_back(completion(&request));
            jit.poll(&memory);
            installed.push((entry, jit.lookup(entry, &memory).unwrap().0));
        }
        let entry = RegionKey {
            pc: 0x70000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 2);
        memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
        jit.sample(entry, 8);
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        assert_eq!(request.regions.len(), 7);
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
            assert_eq!(jit.lookup(*entry, &memory).unwrap().0, *handle);
            assert_eq!(jit.lookup(RegionKey { pc: entry.pc + 2, ..*entry }, &memory).unwrap().0, *handle);
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
    for page in 1..=13 {
        let entry = RegionKey {
            pc: page * 0x10000,
            thumb: true,
            cpu_mode: 0x1f,
        };
        memory.map(entry.pc, 4);
        memory.write_range(entry.pc, &[0xc0, 0x46, 0x70, 0x47]).unwrap();
        jit.sample(entry, 8);
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        if page > 7 {
            let before: Vec<_> = entries.iter().map(|key| jit.lookup(*key, &memory).unwrap().0).collect();
            let old_request = responses.lock().requests[0].clone();
            responses.lock().ready.push_back(completion(&old_request));
            let retired_before = responses.lock().retired.len();
            jit.poll(&memory);
            for (key, handle) in entries.iter().zip(before) {
                assert_eq!(jit.lookup(*key, &memory).unwrap().0, handle);
                assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..*key }, &memory).unwrap().0, handle);
                assert!(!responses.lock().retired[retired_before..].contains(&handle));
            }
            assert!(jit.lookup(entry, &memory).is_none());
        }
        let old_handles: Vec<_> = request.regions.iter().filter_map(|region| region.expected_old).collect();
        for region in &request.regions {
            assert_eq!(jit.lookup(region.ir.entry, &memory).map(|value| value.0), region.expected_old);
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
            assert_eq!(jit.lookup(key, &memory).unwrap().0, handle);
            assert_eq!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory).unwrap().0, handle);
            assert!(!old_handles.contains(&handle));
        }
        entries.push(entry);
    }
    assert_eq!(merges, 2);
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
        jit.sample(entry, 8);
        jit.maintain(&memory);
        let request = responses.lock().requests.last().unwrap().clone();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        installed.push((entry, jit.lookup(entry, &memory).unwrap().0));
    }
    let entry = RegionKey {
        pc: 0x30000,
        ..installed[0].0
    };
    memory.map(entry.pc, 2);
    memory.write_range(entry.pc, &[0x70, 0x47]).unwrap();
    jit.sample(
        RegionKey {
            pc: installed[0].0.pc + 2,
            ..installed[0].0
        },
        9,
    );
    jit.sample(entry, 8);
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
            jit.sample(entry, hits);
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
    let handle = jit.lookup(entry, &memory).unwrap().0;
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
        let code = [0xc0, 0x46].repeat(256); // A full analysis window of nop instructions.
        memory.map(entry.pc, code.len());
        memory.write_range(entry.pc, &code).unwrap();
        jit.sample(entry, 8);
        let before: Vec<_> = entries
            .iter()
            .filter_map(|(key, _)| jit.lookup(*key, &memory).map(|value| (*key, value.0)))
            .collect();
        let retired_before = responses.lock().retired.len();
        jit.maintain(&memory);
        assert_eq!(responses.lock().requests.len(), page as usize);
        let request = responses.lock().requests.last().unwrap().clone();
        let index_bytes: usize = request
            .regions
            .iter()
            .map(|region| {
                region
                    .ir
                    .blocks
                    .iter()
                    .flat_map(|block| &block.instructions)
                    .filter(|instruction| instruction.operation != Operation::Interpret)
                    .count()
                    * core::mem::size_of::<(RegionKey, u32)>()
            })
            .sum();
        assert!(4 * request.ir_size() + 5 * serde_json::to_vec(&request).unwrap().len() + 2 * index_bytes <= 1024 * 1024);
        if page == 7 {
            assert!(
                (2..7).contains(&request.regions.len()),
                "IR budget must limit the otherwise eligible six old regions"
            );
        }
        for (key, handle) in before {
            if jit.lookup(key, &memory).is_none() {
                assert!(responses.lock().retired[retired_before..].contains(&handle));
                assert!(jit.lookup(RegionKey { pc: key.pc + 2, ..key }, &memory).is_none());
                evictions += 1;
            }
        }
        entries.push((entry, request.regions[0].ir_size() + 2 * 256 * core::mem::size_of::<(RegionKey, u32)>()));
        let old_handles: Vec<_> = entries
            .iter()
            .filter_map(|(key, _)| jit.lookup(*key, &memory).map(|value| value.0))
            .collect();
        responses.lock().ready.push_back(completion(&request));
        jit.poll(&memory);
        let handle = jit.lookup(entry, &memory).unwrap().0;
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
    // Request clones trim Vec capacity, so their sizes are a lower bound on retained IR.
    assert!(entries.iter().map(|(_, size)| size).sum::<usize>() > 1024 * 1024);
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
    jit.sample(entry, 8);
    jit.maintain(&memory);
    responses.lock().ready.push_back(CompileCompletion {
        session: 1,
        request: 1,
        result: Err("compile failed".into()),
    });
    jit.poll(&memory);
    for hits in 9..20 {
        jit.sample(entry, hits);
        jit.maintain(&memory);
    }
    assert_eq!(responses.lock().requests.len(), 1);
    memory.write_range(0x1000, &[0x08, 0x47]).unwrap();
    jit.sample(entry, 20);
    jit.maintain(&memory);
    assert_eq!(responses.lock().requests.len(), 2);
}
