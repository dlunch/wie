mod analysis;

use alloc::{boxed::Box, collections::BTreeMap, collections::BTreeSet, collections::VecDeque, vec, vec::Vec};
use core::mem::{size_of, size_of_val};

use wie_arm_jit::{Admission, CodePageStamp, CompileRegion, CompileRequest, CompiledExecutor, CompiledHandle, RegionKey};

use crate::engine::EmulatedMemory;

const RETAINED_IR_BYTES: usize = 1024 * 1024;
const PENDING_IR_BYTES: usize = 1024 * 1024;

struct Translation {
    region: CompileRegion,
    handle: CompiledHandle,
    last_used: u64,
    compile_cost: usize,
}

struct Pending {
    batch: CompileRequest,
    new_cost: usize,
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

    pub fn sample(&mut self, key: RegionKey, hits: u32) {
        self.sequence += 1;
        if let Some(entry) = self.owner(key) {
            self.installed.get_mut(&entry).unwrap().last_used = self.sequence;
        } else if hits >= 8
            && self.candidates.len() < 4
            && !self.candidates.contains(&key)
            && !self
                .pending
                .as_ref()
                .is_some_and(|pending| pending.entries.binary_search_by_key(&key, |(key, _)| *key).is_ok())
        {
            self.candidates.push_back(key);
        }
    }

    pub fn lookup(&mut self, key: RegionKey, memory: &EmulatedMemory) -> Option<(CompiledHandle, [CodePageStamp; 1])> {
        loop {
            let entry = self.owner(key)?;
            let translation = &self.installed[&entry];
            if memory.code_is_current(&translation.region.source) {
                // Analysis stays within one code_snapshot page.
                return Some((translation.handle, [translation.region.source[0]]));
            }
            self.retire(entry);
        }
    }

    fn owner(&self, key: RegionKey) -> Option<RegionKey> {
        // Manifest ownership stays canonical even when translations overlap.
        if self.installed.contains_key(&key) {
            return Some(key);
        }
        self.entries.binary_search_by_key(&key, |(key, _)| *key).ok().map(|index| RegionKey {
            pc: self.entries[index].1,
            ..key
        })
    }

    fn retire(&mut self, entry: RegionKey) {
        let translation = self.installed.remove(&entry).unwrap();
        let mut entries = core::mem::take(&mut self.entries).into_vec();
        entries.retain(|(key, owner)| RegionKey { pc: *owner, ..*key } != entry);
        self.entries = entries.into_boxed_slice();
        self.executor.retire(&[translation.handle]);
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
            batch: request, new_cost, ..
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
                let mut retired = Vec::new();
                for (region, compiled) in request.regions.into_iter().zip(artifact.regions) {
                    let entry = region.ir.entry;
                    let translation = Translation {
                        // A future merge serializes the replacement handle as well.
                        compile_cost: self.installed.get(&entry).map_or(new_cost + 256, |previous| previous.compile_cost),
                        region,
                        handle: compiled.handle,
                        last_used: self.sequence,
                    };
                    if let Some(previous) = self.installed.insert(entry, translation) {
                        retired.push(previous.handle);
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
        let Some(entry) = self.candidates.pop_front() else { return };
        if self.lookup(entry, memory).is_some() || self.failed.get(&entry).is_some_and(|source| memory.code_is_current(source)) {
            return;
        }
        let Some((base, bytes, stamp)) = memory.code_snapshot(entry.pc) else {
            return;
        };
        let Some(ir) = analysis::analyze(&bytes, base, entry) else {
            self.remember_failure(entry, vec![stamp]);
            return;
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
                return;
            }
        };
        let index_bytes = entry_index_bytes(&region);
        let new_cost = 4 * region.ir_size() + 5 * payload.len() + 2 * index_bytes;
        drop(payload);
        let mut request_cost = new_cost + 2048;
        if request_cost > PENDING_IR_BYTES {
            self.remember_failure(entry, region.source);
            return;
        }
        // Reserve both the old index and its replacement during an atomic rebuild.
        let mut retained = self.installed.values().map(|translation| translation.region.ir_size()).sum::<usize>()
            + 2 * size_of_val(self.entries.as_ref())
            + size_of_val(&self.entries);
        while retained + region.ir_size() + 2 * index_bytes > RETAINED_IR_BYTES {
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
            self.retire(key);
        }
        let mut regions = vec![region];
        let generations: BTreeSet<_> = self.installed.values().map(|translation| translation.handle.generation).collect();
        if generations.len() >= 6 {
            let mut hot: Vec<_> = self
                .installed
                .values()
                .filter(|translation| memory.code_is_current(&translation.region.source))
                .collect();
            hot.sort_by_key(|translation| core::cmp::Reverse(translation.last_used));
            for translation in hot.into_iter().take(15) {
                if request_cost + translation.compile_cost > PENDING_IR_BYTES {
                    continue;
                }
                request_cost += translation.compile_cost;
                regions.push(CompileRegion {
                    expected_old: Some(translation.handle),
                    ..translation.region.clone()
                });
            }
        }
        self.next_request += 1;
        let request = CompileRequest {
            session: self.session,
            request: self.next_request,
            regions,
        };
        match self.executor.submit(request.clone()) {
            Admission::Accepted => {
                let entries = entry_index(request.regions.iter());
                self.candidates.retain(|key| entries.binary_search_by_key(key, |(key, _)| *key).is_err());
                self.pending = Some(Pending {
                    batch: request,
                    new_cost,
                    entries,
                });
            }
            Admission::Busy => {
                // Capacity is reclaimed only at the scheduler boundary, never in
                // a per-instruction retry loop.
                if let Some(key) = self
                    .installed
                    .iter()
                    .min_by_key(|(_, translation)| translation.last_used)
                    .map(|(key, _)| *key)
                {
                    self.retire(key);
                    self.candidates.push_front(entry);
                }
            }
            Admission::Failed(error) => {
                tracing::warn!(%error, pc = entry.pc, "ARM JIT request failed");
                self.remember_failure(entry, vec![stamp]);
            }
        }
    }

    fn remember_failure(&mut self, entry: RegionKey, source: Vec<CodePageStamp>) {
        if self.failed.len() == 8192 {
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
    entries.sort_unstable();
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
