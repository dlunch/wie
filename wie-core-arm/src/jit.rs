mod analysis;

use alloc::{boxed::Box, collections::BTreeMap, collections::VecDeque, vec, vec::Vec};
use core::mem::{size_of, size_of_val};

use wie_arm_jit_types::{Admission, CodePageStamp, CompileRegion, CompileRequest, CompiledExecutor, CompiledHandle, RegionKey};

use crate::engine::EmulatedMemory;

const RETAINED_IR_BYTES: usize = 1024 * 1024;
const PENDING_IR_BYTES: usize = 1024 * 1024;
const HOTNESS_THRESHOLD: u32 = 8;
const MAX_CANDIDATES: usize = 4;
const MERGE_THRESHOLD: usize = 14;
const MERGE_GENERATION_COUNT: usize = 2;
const MAX_FAILED_ENTRIES: usize = 8192;

struct Translation {
    region: CompileRegion,
    handle: CompiledHandle,
    last_used: u64,
    compile_cost: usize,
}

struct Pending {
    batch: CompileRequest,
    new_costs: Vec<usize>,
    entries: Box<[(RegionKey, u32)]>,
}

pub(crate) struct Jit {
    pub executor: Box<dyn CompiledExecutor>,
    session: u64,
    next_request: u64,
    installed: BTreeMap<RegionKey, Translation>,
    entries: Box<[(RegionKey, u32)]>,
    candidates: VecDeque<RegionKey>,
    pending: Option<Pending>,
    failed: BTreeMap<RegionKey, Vec<CodePageStamp>>,
    sequence: u64,
}

impl Jit {
    #[cfg(any(test, target_arch = "wasm32"))]
    pub fn new(session: u64, executor: Box<dyn CompiledExecutor>) -> Self {
        Self {
            executor,
            session,
            next_request: 0,
            installed: BTreeMap::new(),
            entries: Box::default(),
            candidates: VecDeque::new(),
            pending: None,
            failed: BTreeMap::new(),
            sequence: 0,
        }
    }

    pub fn sample(&mut self, location: RegionKey, hotness: Option<(RegionKey, u32)>) {
        self.sequence += 1;
        if let Some(entry) = self.owner(location) {
            self.installed.get_mut(&entry).unwrap().last_used = self.sequence;
        }
        if let Some((key, hits)) = hotness
            && hits >= HOTNESS_THRESHOLD
            && self.owner(key).is_none()
            && self.candidates.len() < MAX_CANDIDATES
            && !self.candidates.contains(&key)
            && !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.entries.binary_search_by_key(&key, |(key, _)| *key).is_ok())
        {
            self.candidates.push_back(key);
        }
    }

    pub fn lookup(&mut self, key: RegionKey, memory: &EmulatedMemory) -> Option<CompiledHandle> {
        loop {
            let entry = self.owner(key)?;
            let translation = &self.installed[&entry];
            if memory.code_is_current(&translation.region.source) {
                return Some(translation.handle);
            }
            self.retire(&[entry]);
        }
    }

    fn owner(&self, key: RegionKey) -> Option<RegionKey> {
        self.entries.binary_search_by_key(&key, |(key, _)| *key).ok().map(|index| RegionKey {
            pc: self.entries[index].1,
            ..key
        })
    }

    fn retire(&mut self, entries: &[RegionKey]) {
        let handles: Vec<_> = entries.iter().map(|entry| self.installed.remove(entry).unwrap().handle).collect();
        self.entries = entry_index(self.installed.values().map(|translation| &translation.region));
        self.executor.retire(&handles);
    }

    pub fn poll(&mut self, memory: &EmulatedMemory) {
        let Some(completion) = self.executor.poll() else { return };
        let current = completion.session == self.session && self.pending.as_ref().is_some_and(|pending| pending.batch.request == completion.request);
        if !current {
            if let Ok(artifact) = completion.result {
                let handles: Vec<_> = artifact
                    .regions
                    .iter()
                    .map(|region| region.handle)
                    .filter(|handle| !self.installed.values().any(|translation| translation.handle == *handle))
                    .collect();
                self.executor.retire(&handles);
            }
            return;
        }
        let Pending {
            batch: request, new_costs, ..
        } = self.pending.take().unwrap();
        match completion.result {
            Ok(artifact) => {
                let valid = artifact.regions.len() == request.regions.len()
                    && request.regions.iter().zip(&artifact.regions).all(|(requested, compiled)| {
                        compiled.manifest.entry == requested.ir.entry
                            && compiled.manifest.source == requested.source
                            && compiled.manifest.expected_old == requested.expected_old
                            && memory.code_is_current(&requested.source)
                            && self.installed.get(&requested.ir.entry).map(|region| region.handle) == requested.expected_old
                    });
                if !valid {
                    self.executor
                        .retire(&artifact.regions.iter().map(|region| region.handle).collect::<Vec<_>>());
                    return;
                }
                let new_entries: Vec<_> = request.regions.iter().take(new_costs.len()).map(|region| region.ir.entry).collect();
                let mut retired = Vec::new();
                for (index, (region, compiled)) in request.regions.into_iter().zip(artifact.regions).enumerate() {
                    let entry = region.ir.entry;
                    // A future merge serializes the replacement handle as well.
                    let (compile_cost, last_used) = self.installed.get(&entry).map_or_else(
                        || (new_costs[index] + 256, self.sequence),
                        |previous| (previous.compile_cost, previous.last_used),
                    );
                    let translation = Translation {
                        region,
                        handle: compiled.handle,
                        last_used,
                        compile_cost,
                    };
                    if let Some(previous) = self.installed.insert(entry, translation) {
                        retired.push(previous.handle);
                    }
                }
                // Only newly analyzed regions add coverage; merged exports keep their old IR.
                for new_entry in new_entries {
                    let new_region = &self.installed[&new_entry].region;
                    let mut instructions = Vec::with_capacity(new_region.ir.blocks.iter().map(|block| block.instructions.len()).sum());
                    instructions.extend(new_region.ir.blocks.iter().flat_map(|block| &block.instructions));
                    instructions.sort_unstable_by_key(|instruction| instruction.pc);
                    let covered: Vec<_> = self
                        .installed
                        .iter()
                        .filter(|(entry, translation)| {
                            **entry != new_entry
                                && entry.thumb == new_entry.thumb
                                && entry.cpu_mode == new_entry.cpu_mode
                                && translation.region.source == new_region.source
                                && translation
                                    .region
                                    .ir
                                    .blocks
                                    .iter()
                                    .flat_map(|block| &block.instructions)
                                    .all(|instruction| {
                                        instructions
                                            .binary_search_by_key(&instruction.pc, |instruction| instruction.pc)
                                            .is_ok_and(|index| instructions[index] == instruction)
                                    })
                        })
                        .map(|(entry, _)| *entry)
                        .collect();
                    for entry in covered {
                        retired.push(self.installed.remove(&entry).unwrap().handle);
                    }
                }
                self.entries = entry_index(self.installed.values().map(|translation| &translation.region));
                self.executor.retire(&retired);
                tracing::debug!(
                    request = completion.request,
                    regions = self.installed.len(),
                    bytes = artifact.encoded_size,
                    "ARM JIT installed"
                );
            }
            Err(error) => {
                tracing::warn!(%error, request = completion.request, "ARM JIT compilation failed");
                for region in request.regions {
                    self.remember_failure(region.ir.entry, region.source);
                }
            }
        }
    }

    pub fn maintain(&mut self, memory: &EmulatedMemory) {
        self.poll(memory);
        if self.pending.is_some() {
            return;
        }
        let mut regions: Vec<CompileRegion> = Vec::new();
        let mut new_costs = Vec::new();
        let mut request_cost = 2048;
        let mut new_retained = 0;
        while let Some(entry) = self.candidates.pop_front() {
            if self.lookup(entry, memory).is_some()
                || self.failed.get(&entry).is_some_and(|source| memory.code_is_current(source))
                || regions.iter().any(|region| {
                    region.ir.entry.thumb == entry.thumb
                        && region.ir.entry.cpu_mode == entry.cpu_mode
                        && region
                            .ir
                            .blocks
                            .iter()
                            .flat_map(|block| &block.instructions)
                            .any(|instruction| instruction.pc == entry.pc)
                })
            {
                continue;
            }
            let Some((base, bytes, stamp)) = memory.code_snapshot(entry.pc) else {
                continue;
            };
            let Some(ir) = analysis::analyze(&bytes, base, entry) else {
                self.remember_failure(entry, vec![stamp]);
                continue;
            };
            let region = CompileRegion {
                ir,
                source: vec![stamp],
                expected_old: None,
            };
            let payload = match serde_json::to_vec(&region) {
                Ok(payload) => payload,
                Err(error) => {
                    tracing::warn!(%error, pc = entry.pc, "ARM JIT serialization failed");
                    self.remember_failure(entry, region.source);
                    continue;
                }
            };
            let index_bytes = entry_index_bytes(&region);
            let new_cost = 4 * region.ir_size() + 5 * payload.len() + 2 * index_bytes;
            drop(payload);
            if new_cost + 2048 > PENDING_IR_BYTES {
                self.remember_failure(entry, region.source);
                continue;
            }
            if request_cost + new_cost > PENDING_IR_BYTES {
                self.candidates.push_front(entry);
                break;
            }
            request_cost += new_cost;
            new_retained += region.ir_size() + 2 * index_bytes;
            new_costs.push(new_cost);
            regions.push(region);
        }
        if regions.is_empty() {
            return;
        }
        let stale: Vec<_> = self
            .installed
            .iter()
            .filter(|(_, translation)| !memory.code_is_current(&translation.region.source))
            .map(|(key, _)| *key)
            .collect();
        if !stale.is_empty() {
            self.retire(&stale);
        }
        // Count raw entries for both indexes, including duplicates present during rebuild.
        let mut retained = self
            .installed
            .values()
            .map(|translation| translation.region.ir_size() + 2 * entry_index_bytes(&translation.region))
            .sum::<usize>()
            + size_of_val(&self.entries);
        while retained + new_retained > RETAINED_IR_BYTES {
            let Some(key) = self
                .installed
                .iter()
                .min_by_key(|(_, translation)| translation.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            let previous = &self.installed[&key];
            retained -= previous.region.ir_size() + 2 * entry_index_bytes(&previous.region);
            self.retire(&[key]);
        }
        let mut generations: BTreeMap<_, Vec<_>> = BTreeMap::new();
        for translation in self.installed.values() {
            generations.entry(translation.handle.generation).or_default().push(translation);
        }
        if generations.len() >= MERGE_THRESHOLD {
            let mut groups: Vec<_> = generations
                .into_values()
                .map(|translations| {
                    (
                        translations.iter().map(|translation| translation.compile_cost).sum::<usize>(),
                        translations,
                    )
                })
                .collect();
            groups.sort_by_key(|(cost, _)| *cost);
            // Replacing complete modules frees capacity; the two cheapest avoid
            // repeatedly recompiling a growing accumulator with every new entry.
            for (cost, translations) in groups.into_iter().take(MERGE_GENERATION_COUNT) {
                if request_cost + cost > PENDING_IR_BYTES {
                    break;
                }
                request_cost += cost;
                regions.extend(translations.into_iter().map(|translation| CompileRegion {
                    expected_old: Some(translation.handle),
                    ..translation.region.clone()
                }));
            }
        }
        self.next_request += 1;
        let request = CompileRequest {
            session: self.session,
            request: self.next_request,
            regions,
        };
        match self.executor.submit(&request) {
            Admission::Accepted => {
                let entries = entry_index(request.regions.iter());
                self.candidates.retain(|key| entries.binary_search_by_key(key, |(key, _)| *key).is_err());
                self.pending = Some(Pending {
                    batch: request,
                    new_costs,
                    entries,
                });
            }
            Admission::Busy => {
                // Capacity is reclaimed only at the scheduler boundary, never in
                // a per-instruction retry loop.
                let mut last_used: BTreeMap<u64, u64> = BTreeMap::new();
                for translation in self.installed.values() {
                    let sequence = last_used.entry(translation.handle.generation).or_default();
                    *sequence = (*sequence).max(translation.last_used);
                }
                if let Some((&generation, _)) = last_used.iter().min_by_key(|(_, sequence)| *sequence) {
                    let entries: Vec<_> = self
                        .installed
                        .iter()
                        .filter(|(_, translation)| translation.handle.generation == generation)
                        .map(|(key, _)| *key)
                        .collect();
                    self.retire(&entries);
                }
                for region in request.regions[..new_costs.len()].iter().rev() {
                    self.candidates.push_front(region.ir.entry);
                }
            }
            Admission::Failed(error) => {
                tracing::warn!(%error, "ARM JIT request failed");
                for region in request.regions.into_iter().take(new_costs.len()) {
                    self.remember_failure(region.ir.entry, region.source);
                }
            }
        }
    }

    fn remember_failure(&mut self, entry: RegionKey, source: Vec<CodePageStamp>) {
        if self.failed.len() == MAX_FAILED_ENTRIES {
            self.failed.pop_first();
        }
        self.failed.insert(entry, source);
    }

    pub fn shutdown(&mut self) {
        self.executor.shutdown();
        self.installed.clear();
        self.entries = Box::default();
        self.candidates.clear();
        self.pending = None;
        self.failed.clear();
    }
}

fn entry_index<'a>(regions: impl Iterator<Item = &'a CompileRegion> + Clone) -> Box<[(RegionKey, u32)]> {
    let count = regions
        .clone()
        .map(|region| entry_index_bytes(region) / size_of::<(RegionKey, u32)>())
        .sum();
    let mut entries = Vec::with_capacity(count);
    for region in regions {
        for instruction in region.ir.blocks.iter().flat_map(|block| &block.instructions) {
            entries.push((
                RegionKey {
                    pc: instruction.pc,
                    ..region.ir.entry
                },
                region.ir.entry.pc,
            ));
        }
    }
    // Canonical owners precede aliases; interior ownership is deterministic.
    entries.sort_unstable_by_key(|(key, owner)| (*key, *owner != key.pc, *owner));
    entries.dedup_by_key(|(key, _)| *key);
    entries.into_boxed_slice()
}

fn entry_index_bytes(region: &CompileRegion) -> usize {
    region.ir.blocks.iter().map(|block| block.instructions.len()).sum::<usize>() * size_of::<(RegionKey, u32)>()
}

impl Drop for Jit {
    fn drop(&mut self) {
        self.shutdown();
    }
}
