use alloc::{collections::BTreeMap, vec, vec::Vec};

use wie_arm_jit::RegionKey;
use wie_backend::ProfileSample;

const MAX_HOTNESS_ENTRIES: usize = 8192;
const PROFILE_MAX_STACK: usize = 32;
const PROFILE_FLUSH_INTERVAL: u64 = 1000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Sample {
    pub location: RegionKey,
    pub hotness: Option<(RegionKey, u32)>,
}

pub(super) struct Sampler {
    pub remaining: u32,
    pub profiling: bool,
    pub sequence: u64,
    pending_key: RegionKey,
    pending_entry_pc: u32,
    pending_stack: Option<Vec<u32>>,
    jitter: u32,
    hotness: BTreeMap<RegionKey, (u32, u64)>,
    profile: BTreeMap<Vec<u32>, u64>,
    profile_ready: bool,
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            remaining: 1024,
            profiling: false,
            sequence: 0,
            pending_key: RegionKey {
                pc: 0,
                thumb: false,
                cpu_mode: 0,
            },
            pending_entry_pc: 0,
            pending_stack: None,
            jitter: 0x9e3779b9,
            hotness: BTreeMap::new(),
            profile: BTreeMap::new(),
            profile_ready: false,
        }
    }

    pub fn prepare(&mut self, pc: u32, cpsr: u32, mut r7: u32, entry_pc: u32, mut read: impl FnMut(u32, &mut [u8]) -> bool) {
        self.pending_key = RegionKey {
            pc,
            thumb: cpsr & 0x20 != 0,
            cpu_mode: (cpsr & 0x1f) as u8,
        };
        self.pending_entry_pc = entry_pc;
        self.pending_stack = if self.profiling {
            let mut stack = vec![pc];
            for _ in 0..PROFILE_MAX_STACK {
                let mut frame = [0; 8];
                if !read(r7, &mut frame) {
                    break;
                }
                let previous = u32::from_le_bytes(frame[..4].try_into().unwrap());
                let lr = u32::from_le_bytes(frame[4..].try_into().unwrap());
                if previous == 0 || lr == 0 || lr & 1 == 0 {
                    break;
                }
                stack.push(lr);
                if previous <= r7 {
                    break;
                }
                r7 = previous;
            }
            Some(stack)
        } else {
            None
        };
    }

    pub fn retire(&mut self, instructions: u32) -> Option<Sample> {
        self.remaining -= instructions;
        if self.remaining != 0 {
            return None;
        }
        self.sequence += 1;
        self.jitter ^= self.jitter << 13;
        self.jitter ^= self.jitter >> 17;
        self.jitter ^= self.jitter << 5;
        self.remaining = 896 + self.jitter % 257;

        let hotness = if self.pending_entry_pc == 0 {
            None
        } else {
            let key = RegionKey {
                pc: self.pending_entry_pc,
                ..self.pending_key
            };
            if self.hotness.len() == MAX_HOTNESS_ENTRIES && !self.hotness.contains_key(&key) {
                // ponytail: eviction scans at most 8192 entries; use an LRU index if measured hotness cost warrants it.
                let oldest = self.hotness.iter().min_by_key(|(_, (_, sequence))| sequence).map(|(key, _)| *key);
                if let Some(oldest) = oldest {
                    self.hotness.remove(&oldest);
                }
            }
            let entry = self.hotness.entry(key).or_default();
            entry.0 = entry.0.saturating_add(1);
            entry.1 = self.sequence;
            Some((key, entry.0))
        };

        if let Some(stack) = self.pending_stack.take() {
            *self.profile.entry(stack).or_default() += 1;
            self.profile_ready |= self.sequence.is_multiple_of(PROFILE_FLUSH_INTERVAL);
        }
        Some(Sample {
            location: self.pending_key,
            hotness,
        })
    }

    pub fn take_profile(&mut self, force: bool) -> Vec<ProfileSample> {
        if !force && !self.profile_ready {
            return Vec::new();
        }
        self.profile_ready = false;
        core::mem::take(&mut self.profile)
            .into_iter()
            .map(|(stack, count)| ProfileSample { stack, count })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, collections::BTreeMap, collections::BTreeSet, sync::Arc, vec, vec::Vec};

    use spin::Mutex;

    use crate::ArmCore;

    use super::Sampler;

    #[test]
    fn profiling_consumes_the_same_retired_sample_without_resampling_the_stack() {
        for profiling in [false, true] {
            let mut sampler = Sampler::new();
            sampler.profiling = profiling;
            sampler.remaining = 1;
            let mut reads = 0;
            sampler.prepare(0x1000, 0x3f, 0x2000, 0x1000, |address, buffer| {
                reads += 1;
                if address == 0x2000 {
                    buffer[..4].copy_from_slice(&0x2010u32.to_le_bytes());
                    buffer[4..].copy_from_slice(&0x3001u32.to_le_bytes());
                } else {
                    buffer.fill(0);
                }
                true
            });
            assert_eq!(reads, if profiling { 2 } else { 0 });
            let sample = sampler.retire(1).unwrap();
            assert_eq!(
                (sample.location.pc, sample.location.thumb, sample.location.cpu_mode),
                (0x1000, true, 0x1f)
            );
            assert_eq!(sample.hotness, Some((sample.location, 1)));
            assert_eq!(sampler.sequence, 1);
            let samples = sampler.take_profile(true);
            if profiling {
                assert_eq!(samples.len(), 1);
                assert_eq!(samples[0].stack, vec![0x1000, 0x3001]);
                assert_eq!(samples[0].count, 1);
            } else {
                assert!(samples.is_empty());
            }
        }
    }

    #[test]
    fn unexecuted_boundaries_do_not_advance_the_sampler() {
        let mut sampler = Sampler::new();
        sampler.remaining = 1;
        sampler.prepare(0x1000, 0x3f, 0, 0x8000, |_, _| false);
        assert!(sampler.retire(0).is_none());
        assert_eq!(sampler.sequence, 0);
        assert_eq!(sampler.remaining, 1);
        assert!(sampler.hotness.is_empty());
        sampler.prepare(0x1002, 0x1f, 0, 0x9000, |_, _| false);
        let sample = sampler.retire(1).unwrap();
        assert_eq!(sample.location.pc, 0x1002);
        assert_eq!(sample.hotness.unwrap().0.pc, 0x9000);
        assert_eq!(sampler.hotness.len(), 1);
        assert_eq!(sampler.sequence, 1);
        assert!((896..=1152).contains(&sampler.remaining));
    }

    #[test]
    fn unknown_entries_preserve_profiles_without_reusing_staged_hotness() {
        for profiling in [false, true] {
            let mut sampler = Sampler::new();
            sampler.profiling = profiling;
            sampler.remaining = 1;
            sampler.prepare(0x1000, 0x3f, 0, 0x8000, |_, _| false);
            assert!(sampler.retire(0).is_none());
            sampler.prepare(0x2004, 0x10, 0, 0, |_, _| false);
            let unknown = sampler.retire(1).unwrap();
            assert_eq!(
                (unknown.location.pc, unknown.location.thumb, unknown.location.cpu_mode),
                (0x2004, false, 0x10)
            );
            assert_eq!(unknown.hotness, None);
            assert!(sampler.hotness.is_empty());
            assert_eq!(sampler.sequence, 1);
            assert!(sampler.retire(0).is_none());

            sampler.remaining = 1;
            sampler.prepare(0x3002, 0x3f, 0, 0x3000, |_, _| false);
            let known = sampler.retire(1).unwrap();
            assert_eq!(known.location.pc, 0x3002);
            assert_eq!(
                known.hotness,
                Some((
                    super::RegionKey {
                        pc: 0x3000,
                        ..known.location
                    },
                    1
                ))
            );
            assert_eq!(sampler.hotness.len(), 1);
            assert_eq!(sampler.sequence, 2);
            let profiles = sampler.take_profile(true);
            if profiling {
                assert_eq!(profiles.len(), 2);
                assert_eq!(profiles[0].stack, [0x2004]);
                assert_eq!(profiles[1].stack, [0x3002]);
                assert!(profiles.iter().all(|sample| sample.count == 1));
            } else {
                assert!(profiles.is_empty());
            }
        }
    }

    #[test]
    fn profiling_and_retirement_batches_preserve_jittered_sample_streams() {
        for loop_length in [1024, 2048, 3072, 4096] {
            let replay = |profiling, batch_size: u32| {
                let mut sampler = Sampler::new();
                sampler.profiling = profiling;
                let mut retired = 0;
                let mut events = Vec::new();
                let mut profiles = BTreeMap::new();
                let mut sampled_pcs = BTreeMap::new();
                let mut reads = 0;
                while retired < 1_200_000 {
                    let count = batch_size.min(sampler.remaining).min(1_200_000 - retired);
                    assert!(sampler.retire(0).is_none());
                    if count == sampler.remaining {
                        // Select the input instruction, not a separately computed sampling schedule.
                        let position = retired + count - 1;
                        let (base, cpsr, width) = if (position / 4096).is_multiple_of(2) {
                            (0x10000, 0x3f, 2)
                        } else {
                            (0x20000, 0x10, 4)
                        };
                        sampler.prepare(base + position % loop_length * width, cpsr, 0, base, |_, _| {
                            reads += 1;
                            false
                        });
                    }
                    retired += count;
                    if let Some(sample) = sampler.retire(count) {
                        events.push((retired, sampler.sequence, sample.location, sample.hotness));
                        *sampled_pcs.entry(vec![sample.location.pc]).or_insert(0u64) += 1;
                        for sample in sampler.take_profile(false) {
                            *profiles.entry(sample.stack).or_insert(0u64) += sample.count;
                        }
                    }
                }
                for sample in sampler.take_profile(true) {
                    *profiles.entry(sample.stack).or_insert(0u64) += sample.count;
                }
                assert!(sampler.sequence > 1000);
                assert_eq!(events[0].0, 1024);
                assert_eq!(reads, if profiling { sampler.sequence } else { 0 });
                if profiling {
                    assert_eq!(profiles, sampled_pcs);
                } else {
                    assert!(profiles.is_empty());
                }
                let phases: BTreeSet<_> = events
                    .iter()
                    .map(|(_, _, key, _)| (key.pc - if key.thumb { 0x10000 } else { 0x20000 }) / if key.thumb { 2 } else { 4 } % 1024)
                    .collect();
                assert!(phases.len() > 1, "fixed-interval aliasing for loop length {loop_length}");
                assert_eq!(
                    events.iter().map(|(_, _, key, _)| (key.thumb, key.cpu_mode)).collect::<BTreeSet<_>>(),
                    BTreeSet::from([(false, 0x10), (true, 0x1f)])
                );
                (events, sampler.remaining, sampler.hotness)
            };
            let expected = replay(false, 1);
            for profiling in [false, true] {
                for batch_size in [1, 127, 4096] {
                    assert_eq!(
                        replay(profiling, batch_size),
                        expected,
                        "loop={loop_length}, profile={profiling}, batch={batch_size}"
                    );
                }
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn run_and_thread_context_boundaries_preserve_the_retired_sample_stream() {
        for switch_threads in [false, true] {
            let observed = Arc::new(Mutex::new(BTreeMap::new()));
            let output = observed.clone();
            let mut core = ArmCore::new(
                false,
                Some(Box::new(move |batch| {
                    for sample in batch {
                        *output.lock().entry(sample.stack).or_insert(0u64) += sample.count;
                    }
                })),
            )
            .unwrap();
            crate::Allocator::init(&mut core).unwrap();
            let threads = [
                core.run_in_thread(|| async { Ok(()) }).unwrap(),
                core.run_in_thread(|| async { Ok(()) }).unwrap(),
            ];
            let thread_ids = core.get_thread_ids();
            for (index, length) in [1024, 2048, 3072, 4096].into_iter().enumerate() {
                let mut code = [0xc0, 0x46].repeat(length - 1); // nop
                code.extend_from_slice(&[0x70, 0x47]); // bx lr
                core.load(&code, 0x10000 + index as u32 * 0x10000, code.len()).unwrap();
            }
            let mut reference = Sampler::new();
            reference.profiling = true;
            for run in 0..512 {
                let length = [1024, 2048, 3072, 4096][run % 4];
                let base = 0x10000 + (run % 4) as u32 * 0x10000;
                let _guard = switch_threads.then(|| core.enter_thread_context(thread_ids[run % 2]));
                futures::executor::block_on(core.run_function::<()>(base | 1, &[])).unwrap();
                for instruction in 0..length {
                    if reference.remaining == 1 {
                        reference.prepare(base + instruction * 2, 0x3f, 0, base, |_, _| false);
                    }
                    reference.retire(1);
                }
            }
            core.shutdown();
            let expected: BTreeMap<_, _> = reference
                .take_profile(true)
                .into_iter()
                .map(|sample| (sample.stack, sample.count))
                .collect();
            assert!(reference.sequence > 1000);
            assert_eq!(*observed.lock(), expected, "switch_threads={switch_threads}");
            drop(threads);
        }
    }
}
