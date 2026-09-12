use alloc::{collections::BTreeMap, vec, vec::Vec};

use wie_backend::ProfileSample;

const PROFILE_MAX_STACK: usize = 32;
const PROFILE_FLUSH_INTERVAL: u64 = 1000;

pub(super) struct Sampler {
    pub remaining: u32,
    pub profiling: bool,
    pub sequence: u64,
    pending_stack: Option<Vec<u32>>,
    jitter: u32,
    profile: BTreeMap<Vec<u32>, u64>,
    profile_ready: bool,
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            remaining: 1024,
            profiling: false,
            sequence: 0,
            pending_stack: None,
            jitter: 0x9e3779b9,
            profile: BTreeMap::new(),
            profile_ready: false,
        }
    }

    pub fn prepare(&mut self, pc: u32, mut r7: u32, mut read: impl FnMut(u32, &mut [u8]) -> bool) {
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

    pub fn retire(&mut self, instructions: u32) -> bool {
        self.remaining -= instructions;
        if self.remaining != 0 {
            return false;
        }
        self.sequence += 1;
        self.jitter ^= self.jitter << 13;
        self.jitter ^= self.jitter >> 17;
        self.jitter ^= self.jitter << 5;
        self.remaining = 896 + self.jitter % 257;

        if let Some(stack) = self.pending_stack.take() {
            *self.profile.entry(stack).or_default() += 1;
            self.profile_ready |= self.sequence.is_multiple_of(PROFILE_FLUSH_INTERVAL);
        }
        true
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
    fn profiling_consumes_the_retired_sample_without_resampling_the_stack() {
        for profiling in [false, true] {
            let mut sampler = Sampler::new();
            sampler.profiling = profiling;
            sampler.remaining = 1;
            let mut reads = 0;
            sampler.prepare(0x1000, 0x2000, |address, buffer| {
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
            assert!(sampler.retire(1));
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
    fn unexecuted_boundaries_do_not_commit_a_profile_sample() {
        let mut sampler = Sampler::new();
        sampler.profiling = true;
        sampler.remaining = 1;
        sampler.prepare(0x1000, 0, |_, _| false);
        assert!(!sampler.retire(0));
        assert_eq!(sampler.sequence, 0);
        assert_eq!(sampler.remaining, 1);
        assert!(sampler.take_profile(true).is_empty());
        sampler.prepare(0x2004, 0, |_, _| false);
        assert!(sampler.retire(1));
        assert_eq!(sampler.sequence, 1);
        assert!((896..=1152).contains(&sampler.remaining));
        let samples = sampler.take_profile(true);
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].stack, [0x2004]);
        assert_eq!(samples[0].count, 1);
        assert!(!sampler.retire(0));
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
                    assert!(!sampler.retire(0));
                    let position = retired + count - 1;
                    let (base, width) = if (position / 4096).is_multiple_of(2) {
                        (0x10000, 2)
                    } else {
                        (0x20000, 4)
                    };
                    let pc = base + position % loop_length * width;
                    if count == sampler.remaining {
                        sampler.prepare(pc, 0, |_, _| {
                            reads += 1;
                            false
                        });
                    }
                    retired += count;
                    if sampler.retire(count) {
                        events.push((retired, sampler.sequence, pc));
                        *sampled_pcs.entry(vec![pc]).or_insert(0u64) += 1;
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
                let phases: BTreeSet<_> = events.iter().map(|(_, _, pc)| pc % 1024).collect();
                assert!(phases.len() > 1, "fixed-interval aliasing for loop length {loop_length}");
                (events, sampler.remaining)
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
                        reference.prepare(base + instruction * 2, 0, |_, _| false);
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
