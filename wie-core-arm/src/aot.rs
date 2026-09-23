mod analysis;
mod decoder;

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::{
    ops::Range,
    task::{Context, Poll, Waker},
};

use hashbrown::HashMap;
use wie_arm_jit_types::{
    CompileRequest, CompiledArtifact, CompiledExecutor, CompiledHandle, CompiledRegion, ManifestRegion, PreparationFuture, PreparationState,
    RegionKey,
};
use wie_util::Result as WieResult;

use crate::engine::EmulatedMemory;

// Bound synchronous analysis/code generation and Wasm selector nesting, including coalesced regions.
// These are policy limits, not ISA limits; larger regions trade fewer dispatches for longer preparation steps.
const MAX_REGION_INSTRUCTIONS: usize = 4096;
const MAX_REGION_BLOCKS: usize = 512;
const PREPARATION_TIMEOUT_MS: f64 = 10_000.0;

struct Recompilation {
    manifest: ManifestRegion,
    future: PreparationFuture,
    invalidated: bool,
}

pub(crate) struct Aot {
    pub executor: Box<dyn CompiledExecutor>,
    pub state: PreparationState,
    pub entries: HashMap<RegionKey, CompiledHandle>,
    ranges: Vec<Range<u64>>,
    regions: Vec<CompiledRegion>,
    invalidated: Vec<ManifestRegion>,
    recompilations: Vec<Recompilation>,
}

impl Aot {
    pub fn new(executor: Box<dyn CompiledExecutor>) -> Self {
        Self {
            executor,
            state: PreparationState::Loading,
            entries: HashMap::new(),
            ranges: Vec::new(),
            regions: Vec::new(),
            invalidated: Vec::new(),
            recompilations: Vec::new(),
        }
    }

    pub fn record_image(&mut self, address: u32, size: usize) {
        if self.state == PreparationState::Loading && size != 0 {
            self.ranges.push(u64::from(address)..u64::from(address) + size as u64);
        }
    }

    pub fn begin(&mut self, memory: &EmulatedMemory) -> WieResult<Option<PreparationFuture>> {
        if self.state != PreparationState::Loading {
            return Ok(None);
        }
        let request = compile_request(memory, core::mem::take(&mut self.ranges), None)?;
        if request.images.is_empty() {
            self.state = PreparationState::Ready;
            return Ok(None);
        }
        let deadline_ms = self.executor.now() + PREPARATION_TIMEOUT_MS;
        self.state = PreparationState::Preparing;
        Ok(Some(self.executor.prepare(request, deadline_ms)))
    }

    pub fn finish(&mut self, result: Result<CompiledArtifact, String>) -> bool {
        if self.state != PreparationState::Preparing {
            return false;
        }
        self.state = PreparationState::Ready;
        match result {
            Ok(artifact) => self.install(artifact),
            Err(error) => {
                tracing::warn!(%error, "ARM AOT preparation failed; using interpreter");
                return false;
            }
        }
        tracing::info!("ARM AOT installed");
        true
    }

    fn install(&mut self, artifact: CompiledArtifact) {
        for region in artifact.regions {
            self.entries.extend(
                region
                    .manifest
                    .instruction_pcs
                    .iter()
                    .map(|&pc| (RegionKey { pc, ..region.manifest.entry }, region.handle)),
            );
            self.regions.push(region);
        }
    }

    pub fn invalidate(&mut self, range: Range<u64>) {
        let overlaps = |manifest: &ManifestRegion| manifest.code_ranges.iter().any(|code| code.start < range.end && range.start < code.end);
        let mut index = 0;
        while index < self.regions.len() {
            if !overlaps(&self.regions[index].manifest) {
                index += 1;
                continue;
            }
            let region = self.regions.swap_remove(index);
            for &pc in &region.manifest.instruction_pcs {
                self.entries.remove(&RegionKey { pc, ..region.manifest.entry });
            }
            self.executor.release(region.handle);
            self.invalidated.push(region.manifest);
        }
        for pending in &mut self.recompilations {
            pending.invalidated |= overlaps(&pending.manifest);
        }
    }

    pub fn recompile(&mut self, key: &RegionKey, memory: &EmulatedMemory) {
        let Some(index) = self.invalidated.iter().position(|region| {
            region.entry.thumb == key.thumb
                && region.entry.cpu_mode == key.cpu_mode
                && region.code_ranges.iter().any(|range| range.contains(&u64::from(key.pc)))
        }) else {
            return;
        };
        let manifest = self.invalidated.swap_remove(index);
        let request = match compile_request(memory, manifest.code_ranges.clone(), Some(manifest.entry.thumb)) {
            Ok(request) => request,
            Err(error) => {
                tracing::warn!(%error, "ARM AOT recompilation failed; using interpreter");
                return;
            }
        };
        let deadline_ms = self.executor.now() + PREPARATION_TIMEOUT_MS;
        self.recompilations.push(Recompilation {
            manifest,
            future: self.executor.prepare(request, deadline_ms),
            invalidated: false,
        });
    }

    pub fn poll_recompilations(&mut self) {
        let mut index = 0;
        while index < self.recompilations.len() {
            let Poll::Ready(result) = self.recompilations[index].future.as_mut().poll(&mut Context::from_waker(Waker::noop())) else {
                index += 1;
                continue;
            };
            let pending = self.recompilations.swap_remove(index);
            if pending.invalidated {
                // Another explicit invalidation supersedes the in-flight snapshot.
                if let Ok(artifact) = result {
                    for region in artifact.regions {
                        self.executor.release(region.handle);
                    }
                }
                self.invalidated.push(pending.manifest);
            } else {
                match result {
                    Ok(artifact) => self.install(artifact),
                    Err(error) => tracing::warn!(%error, "ARM AOT recompilation failed; using interpreter"),
                }
            }
        }
    }
}

fn compile_request(memory: &EmulatedMemory, mut ranges: Vec<Range<u64>>, mode: Option<bool>) -> WieResult<CompileRequest> {
    ranges.sort_unstable_by_key(|range| range.start);
    let mut merged: Vec<Range<u64>> = Vec::new();
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && range.start <= previous.end
        {
            previous.end = previous.end.max(range.end);
        } else {
            merged.push(range);
        }
    }
    let mut images = Vec::new();
    for range in merged {
        let mut address = range.start;
        while address < range.end {
            let size = (0x4000 - (address & 0x3fff)).min(range.end - address) as usize;
            images.push(memory.code_image(address as u32, size)?);
            address += size as u64;
        }
    }
    let images: Arc<[_]> = images.into();
    Ok(CompileRequest {
        images: images.clone(),
        max_region_instructions: MAX_REGION_INSTRUCTIONS,
        max_region_blocks: MAX_REGION_BLOCKS,
        regions: Box::new(decoder::Decoder::new(images, mode)),
    })
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, collections::BTreeSet, sync::Arc, vec, vec::Vec};

    use wie_arm_jit_types::{CodeImage, CompileRequest};
    use wie_core_arm_wasm::compile;

    use super::{MAX_REGION_BLOCKS, MAX_REGION_INSTRUCTIONS, decoder::Decoder};

    fn image(address: u32, bytes: Vec<u8>) -> CodeImage {
        CodeImage { address, bytes }
    }

    #[test]
    fn raw_images_stream_into_one_module_with_exact_ownership() {
        let images = vec![
            // ARM countdown, unsupported SVC, and a later branch into an owned instruction.
            image(
                0x1000,
                [0xe3a00003_u32, 0xe2500001, 0xe3500000, 0x1afffffc, 0xe12fff1e, 0xef000000, 0xeafffff9]
                    .into_iter()
                    .flat_map(u32::to_le_bytes)
                    .collect(),
            ),
            // Thumb countdown, unsupported SVC, a later owned target, and a truncated BL prefix.
            image(
                0x2000,
                [0x2003_u16, 0x3801, 0x2800, 0xd1fc, 0x4770, 0xdf00, 0xe7f9, 0xf000]
                    .into_iter()
                    .flat_map(u16::to_le_bytes)
                    .collect(),
            ),
            // One region crosses a backing-memory page boundary.
            image(0xfffe, [0x3001_u16, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect()),
            image(0x20000, [0xf000_u16, 0xf800, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect()),
        ];
        let images: Arc<[_]> = images.into();
        let artifact = compile(CompileRequest {
            images: images.clone(),
            max_region_instructions: MAX_REGION_INSTRUCTIONS,
            max_region_blocks: MAX_REGION_BLOCKS,
            regions: Box::new(Decoder::new(images, None)),
        })
        .unwrap();
        assert_eq!(&artifact.bytes[..8], b"\0asm\x01\0\0\0");
        let mut owned = BTreeSet::new();
        for region in &artifact.manifest {
            assert_eq!(region.entry.cpu_mode, 0x1f);
            assert!(region.instruction_pcs.contains(&region.entry.pc));
            assert!(region.instruction_pcs.windows(2).all(|pcs| pcs[0] < pcs[1]));
            for &pc in &region.instruction_pcs {
                assert!(owned.insert((region.entry.thumb, pc)), "duplicate ownership at {pc:#x}");
            }
        }
        for (thumb, hole, root) in [(false, 0x1014, 0x1018), (true, 0x200a, 0x200c)] {
            assert!(!owned.contains(&(thumb, hole)));
            assert!(owned.contains(&(thumb, root)));
        }
        assert!(!owned.contains(&(true, 0x200e)));
        assert_eq!(
            artifact.manifest.iter().find(|region| region.entry.pc == 0xfffe).unwrap().code_ranges,
            core::iter::once(0xfffe..0x10002).collect::<Vec<_>>()
        );
    }

    #[test]
    fn partial_instructions_remain_fallback_holes() {
        for bytes in [vec![], vec![1], vec![0, 0xf0], vec![0, 0xf0, 0]] {
            assert!(Decoder::new(vec![image(0x1000, bytes)].into(), None).flatten().next().is_none());
        }
        let regions: Vec<_> = Decoder::new(vec![image(0x1001, vec![0xff, 1, 0x20, 0])].into(), None).flatten().collect();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].ir.entry.pc, 0x1002);
        assert_eq!(regions[0].ir.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn unsupported_images_make_bounded_progress_without_ir() {
        let mut decoder = Decoder::new(vec![image(0x1000, vec![0xff; 16 * 1024])].into(), None);
        let mut steps = 0;
        for region in decoder.by_ref() {
            assert!(region.is_none());
            steps += 1;
        }
        assert!(steps >= (16 * 1024 / 4 + 16 * 1024 / 2) / 256);
        assert!(decoder.next().is_none());
    }

    #[test]
    fn analysis_limits_only_claim_emitted_instructions() {
        for code in [vec![0x3001_u16; MAX_REGION_INSTRUCTIONS + 1], vec![0xd1ff_u16; MAX_REGION_BLOCKS + 1]] {
            let images: Arc<[_]> = vec![image(0x1000, code.iter().flat_map(|op| op.to_le_bytes()).collect())].into();
            let artifact = compile(CompileRequest {
                images: images.clone(),
                max_region_instructions: MAX_REGION_INSTRUCTIONS,
                max_region_blocks: MAX_REGION_BLOCKS,
                regions: Box::new(Decoder::new(images, None)),
            })
            .unwrap();
            let mut pcs = BTreeSet::new();
            let regions: Vec<_> = artifact.manifest.iter().filter(|region| region.entry.thumb).collect();
            assert!(regions.len() > 1);
            for region in regions {
                assert!(region.instruction_pcs.len() <= MAX_REGION_INSTRUCTIONS);
                for &pc in &region.instruction_pcs {
                    assert!(pcs.insert(pc));
                }
            }
            assert_eq!(pcs, (0..code.len()).map(|index| 0x1000 + index as u32 * 2).collect());
        }
    }
}
