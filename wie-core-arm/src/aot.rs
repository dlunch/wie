mod analysis;
mod decoder;
mod dummy_executor;

#[cfg(test)]
mod tests;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{cell::RefCell, ops::Range};

use hashbrown::HashMap;
use wie_arm_jit_types::{
    CodePageStamp, CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, PreparationFuture, PreparationState, RegionKey,
};
use wie_util::Result;

use crate::engine::EmulatedMemory;

pub(crate) use dummy_executor::DummyExecutor;

struct Translation {
    source: RefCell<Vec<CodePageStamp>>,
    source_bytes: Vec<(u32, Vec<u8>)>,
    handle: CompiledHandle,
}

pub(crate) struct Aot {
    pub executor: Box<dyn CompiledExecutor>,
    pub state: PreparationState,
    ranges: Vec<Range<u64>>,
    translations: Vec<Option<Translation>>,
    entries: HashMap<RegionKey, CompiledHandle>,
    deadline_ms: f64,
}

impl Aot {
    pub fn new(executor: Box<dyn CompiledExecutor>) -> Self {
        Self {
            executor,
            state: PreparationState::Loading,
            ranges: Vec::new(),
            translations: Vec::new(),
            entries: HashMap::new(),
            deadline_ms: 0.0,
        }
    }

    pub fn record_image(&mut self, address: u32, size: usize) {
        if self.state == PreparationState::Loading && size != 0 {
            self.ranges.push(u64::from(address)..u64::from(address) + size as u64);
        }
    }

    pub fn begin(&mut self, memory: &EmulatedMemory, now: impl Fn() -> f64) -> Result<Option<PreparationFuture>> {
        if self.state != PreparationState::Loading {
            return Ok(None);
        }
        let snapshot_started = now();
        self.ranges.sort_unstable_by_key(|range| range.start);
        let mut ranges: Vec<Range<u64>> = Vec::new();
        for range in core::mem::take(&mut self.ranges) {
            if let Some(previous) = ranges.last_mut()
                && range.start <= previous.end
            {
                previous.end = previous.end.max(range.end);
            } else {
                ranges.push(range);
            }
        }
        let mut images = Vec::new();
        for range in ranges {
            let mut address = range.start;
            while address < range.end {
                let size = (0x4000 - (address & 0x3fff)).min(range.end - address) as usize;
                images.push(memory.code_image(address as u32, size)?);
                address += size as u64;
            }
        }
        if images.is_empty() {
            self.state = PreparationState::Ready;
            return Ok(None);
        }
        let started = now();
        self.deadline_ms = started + 10_000.0;
        tracing::info!(
            bytes = images.iter().map(|image| image.bytes.len()).sum::<usize>(),
            images = images.len(),
            snapshot_ms = started - snapshot_started,
            "ARM AOT input ready"
        );
        self.state = PreparationState::Preparing;
        let images: Arc<[_]> = images.into();
        let request = CompileRequest {
            images: images.clone(),
            regions: Box::new(decoder::Decoder::new(images)),
        };
        Ok(Some(self.executor.prepare(request, self.deadline_ms)))
    }

    pub fn finish(&mut self, result: core::result::Result<CompiledArtifact, alloc::string::String>, memory: &EmulatedMemory, now: impl Fn() -> f64) {
        if self.state != PreparationState::Preparing {
            return;
        }
        if now() >= self.deadline_ms {
            tracing::warn!("ARM AOT preparation timed out; using interpreter");
            self.shutdown();
            return;
        }
        self.state = PreparationState::Ready;
        let mut artifact = match result {
            Ok(artifact) => artifact,
            Err(error) => {
                tracing::warn!(%error, "ARM AOT preparation failed; using interpreter");
                self.shutdown();
                return;
            }
        };
        if artifact
            .regions
            .iter_mut()
            .any(|region| !memory.validate_code(&mut region.manifest.source, &region.manifest.source_bytes))
        {
            self.shutdown();
            return;
        }
        let mut entries = HashMap::with_capacity(artifact.regions.iter().map(|region| region.manifest.instruction_pcs.len()).sum());
        let mut translations = Vec::with_capacity(artifact.regions.len());
        for region in artifact.regions {
            entries.extend(
                region
                    .manifest
                    .instruction_pcs
                    .into_iter()
                    .map(|pc| (RegionKey { pc, ..region.manifest.entry }, region.handle)),
            );
            translations.push(Some(Translation {
                source: RefCell::new(region.manifest.source),
                source_bytes: region.manifest.source_bytes,
                handle: region.handle,
            }));
        }
        // Installation shares the compiler's deadline, including index construction.
        let finished = now();
        if finished >= self.deadline_ms {
            tracing::warn!("ARM AOT installation timed out; using interpreter");
            self.shutdown();
            return;
        }
        self.entries = entries;
        self.translations = translations;
        tracing::info!(
            regions = self.translations.len(),
            bytes = artifact.encoded_size,
            elapsed_ms = finished - (self.deadline_ms - 10_000.0),
            "ARM AOT installed"
        );
    }

    pub fn lookup(&mut self, key: RegionKey, memory: &EmulatedMemory) -> Option<CompiledHandle> {
        let handle = *self.entries.get(&key)?;
        let translation = self.translations[handle.slot as usize].as_ref()?;
        if memory.validate_code(&mut translation.source.borrow_mut(), &translation.source_bytes) {
            return Some(handle);
        }
        self.executor.retire(core::slice::from_ref(&translation.handle));
        self.translations[handle.slot as usize] = None;
        None
    }

    pub fn execution_parts(
        &mut self,
    ) -> (
        &mut dyn CompiledExecutor,
        impl Fn(RegionKey, &EmulatedMemory) -> Option<CompiledHandle> + '_,
    ) {
        let entries = &self.entries;
        let translations = &self.translations;
        (&mut *self.executor, move |key, memory| {
            let handle = *entries.get(&key)?;
            let translation = translations[handle.slot as usize].as_ref()?;
            memory
                .validate_code(&mut translation.source.borrow_mut(), &translation.source_bytes)
                .then_some(handle)
        })
    }

    pub fn shutdown(&mut self) {
        self.state = PreparationState::Ready;
        self.executor.shutdown();
        self.ranges.clear();
        self.translations.clear();
        self.entries = HashMap::new();
    }
}

impl Drop for Aot {
    fn drop(&mut self) {
        self.shutdown();
    }
}
