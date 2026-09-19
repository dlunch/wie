#![no_std]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};

use nom::{
    Parser,
    multi::{length_count, length_data},
    number::complete::le_u32,
};

use wie_arm_jit_types::{CodeImage, CompileRegion, CompileRequest, ManifestRegion, RegionKey};

mod codegen;

#[cfg(target_arch = "wasm32")]
mod runtime;

#[cfg(target_arch = "wasm32")]
pub use runtime::{WasmExecutor, now};

const COPY_SIZE: usize = 64 * 1024;

/// Increment when analysis, generated code, execution ABI, or the cache format changes.
pub const AOT_CACHE_VERSION: u32 = 1;

pub fn encode_manifest_region(region: &ManifestRegion, output: &mut Vec<u8>) {
    // Little-endian header, instruction PCs, then length-prefixed source spans; stamps are session-local.
    for word in [
        region.entry.pc,
        u32::from(region.entry.cpu_mode) << 1 | u32::from(region.entry.thumb),
        region.instruction_pcs.len() as u32,
    ]
    .into_iter()
    .chain(region.instruction_pcs.iter().copied())
    {
        output.extend_from_slice(&word.to_le_bytes());
    }
    output.extend_from_slice(&(region.source_bytes.len() as u32).to_le_bytes());
    for (address, bytes) in &region.source_bytes {
        output.extend_from_slice(&address.to_le_bytes());
        output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        output.extend_from_slice(bytes);
    }
}

pub fn decode_manifest_region(input: &mut &[u8]) -> Option<ManifestRegion> {
    let decoded: nom::IResult<_, _> = (
        le_u32,
        le_u32,
        length_count(le_u32, le_u32),
        length_count(le_u32, (le_u32, length_data(le_u32))),
    )
        .parse(*input);
    let (remaining, (pc, mode, instruction_pcs, spans)) = decoded.ok()?;
    let entry = RegionKey {
        pc,
        thumb: mode & 1 != 0,
        cpu_mode: u8::try_from(mode >> 1).ok()?,
    };
    *input = remaining;
    Some(ManifestRegion {
        entry,
        instruction_pcs,
        source: Vec::new(),
        source_bytes: spans.into_iter().map(|(address, bytes)| (address, bytes.to_vec())).collect(),
    })
}

pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub manifest: Vec<ManifestRegion>,
}

pub struct Compiler {
    request: CompileRequest,
    builder: Option<codegen::ModuleBuilder>,
    pending: Option<CompileRegion>,
    pending_instructions: usize,
    pending_page: Option<u32>,
    chunks: VecDeque<Vec<u8>>,
    chunk_offset: usize,
    artifact: WasmArtifact,
    complete: bool,
}

impl Compiler {
    pub fn new(request: CompileRequest) -> Self {
        Self {
            request,
            builder: Some(codegen::ModuleBuilder::default()),
            pending: None,
            pending_instructions: 0,
            pending_page: None,
            chunks: VecDeque::new(),
            chunk_offset: 0,
            artifact: WasmArtifact {
                bytes: Vec::new(),
                manifest: Vec::new(),
            },
            complete: false,
        }
    }

    /// Consumes one bounded decoder step or assembles at most 64 KiB.
    pub fn step(&mut self) -> Result<bool, String> {
        if self.complete {
            return Ok(true);
        }
        if let Some(builder) = &mut self.builder {
            let next = self.request.regions.next();
            let finished = next.is_none();
            let region = match next {
                Some(None) => return Ok(false),
                Some(Some(mut region)) => {
                    let (instructions, first, last) = region
                        .ir
                        .blocks
                        .iter()
                        .flat_map(|block| &block.instructions)
                        .fold((0, u32::MAX, 0), |(count, first, last), instruction| {
                            (count + 1, first.min(instruction.pc), last.max(instruction.pc))
                        });
                    // Coalesce small fragments without expanding the selectors of larger regions.
                    let page = (instructions <= 16 && first >> 14 == last >> 14).then_some(first >> 14);
                    if let Some(pending) = &mut self.pending
                        && page.is_some()
                        && page == self.pending_page
                        && pending.ir.entry.thumb == region.ir.entry.thumb
                        && pending.ir.entry.cpu_mode == region.ir.entry.cpu_mode
                        && self.pending_instructions + instructions <= 512
                        && pending.ir.blocks.len() + region.ir.blocks.len() <= 128
                    {
                        self.pending_instructions += instructions;
                        pending.ir.blocks.append(&mut region.ir.blocks);
                        pending.source.append(&mut region.source);
                        pending.source_bytes.append(&mut region.source_bytes);
                        return Ok(false);
                    }
                    self.pending_instructions = instructions;
                    self.pending_page = page;
                    self.pending.replace(region)
                }
                None => self.pending.take(),
            };
            if let Some(mut region) = region {
                region.source.sort_unstable_by_key(|stamp| stamp.page);
                region.source.dedup_by_key(|stamp| stamp.page);
                builder.add_region(&region.ir);
                let mut instruction_pcs: Vec<_> = region
                    .ir
                    .blocks
                    .iter()
                    .flat_map(|block| &block.instructions)
                    .map(|instruction| instruction.pc)
                    .collect();
                instruction_pcs.sort_unstable();
                self.artifact.manifest.push(ManifestRegion {
                    entry: region.ir.entry,
                    instruction_pcs,
                    source: region.source,
                    source_bytes: region.source_bytes,
                });
            }
            if finished {
                let builder = core::mem::take(builder);
                self.builder = None;
                self.chunks = builder.begin_assembly(&mut self.artifact.bytes)?;
            }
            return Ok(false);
        }

        let mut remaining = COPY_SIZE;
        while remaining != 0 {
            let Some(chunk) = self.chunks.front() else {
                self.complete = true;
                return Ok(true);
            };
            let count = remaining.min(chunk.len() - self.chunk_offset);
            self.artifact
                .bytes
                .extend_from_slice(&chunk[self.chunk_offset..self.chunk_offset + count]);
            self.chunk_offset += count;
            remaining -= count;
            if self.chunk_offset == chunk.len() {
                self.chunks.pop_front();
                self.chunk_offset = 0;
            }
        }
        Ok(false)
    }

    /// Call after `step` returns true. Only moves the completed output; it performs no compilation or serialization.
    pub fn finish(self) -> WasmArtifact {
        self.artifact
    }
}

pub fn compile(request: CompileRequest) -> Result<WasmArtifact, String> {
    let mut compiler = Compiler::new(request);
    while !compiler.step()? {}
    Ok(compiler.finish())
}

/// Validates cached coverage against immutable inputs before granting current page stamps.
pub fn bind_manifest_source(region: &mut ManifestRegion, images: &[CodeImage]) -> Result<(), String> {
    let alignment = if region.entry.thumb { 2 } else { 4 };
    if region.entry.cpu_mode != 0x1f
        || region.instruction_pcs.binary_search(&region.entry.pc).is_err()
        || region.instruction_pcs.windows(2).any(|pcs| pcs[0] >= pcs[1])
        || region.instruction_pcs.iter().any(|pc| !pc.is_multiple_of(alignment))
        || region.source_bytes.is_empty()
    {
        return Err(String::from("invalid cached instruction coverage"));
    }
    let mut source = Vec::new();
    for (address, bytes) in &region.source_bytes {
        let end = u64::from(*address).checked_add(bytes.len() as u64);
        if bytes.is_empty()
            || end.is_none_or(|end| end > u64::from(u32::MAX) + 1)
            || !address.is_multiple_of(alignment)
            || !bytes.len().is_multiple_of(alignment as usize)
            || region.instruction_pcs.binary_search(address).is_err()
        {
            return Err(String::from("invalid cached source span"));
        }
        let mut address = u64::from(*address);
        let mut remaining = bytes.as_slice();
        while !remaining.is_empty() {
            let index = images.partition_point(|image| u64::from(image.address) <= address);
            let image = index
                .checked_sub(1)
                .and_then(|index| images.get(index))
                .ok_or_else(|| String::from("cached source is outside input"))?;
            let offset = (address - u64::from(image.address)) as usize;
            let input = image
                .bytes
                .get(offset..)
                .filter(|bytes| !bytes.is_empty())
                .ok_or_else(|| String::from("cached source is outside input"))?;
            let size = remaining.len().min(input.len());
            if input[..size] != remaining[..size] {
                return Err(String::from("cached source differs from input"));
            }
            source.extend(
                image
                    .source
                    .iter()
                    .filter(|stamp| u64::from(stamp.page) >= (address & !0xffff) && u64::from(stamp.page) <= ((address + size as u64 - 1) & !0xffff))
                    .copied(),
            );
            address += size as u64;
            remaining = &remaining[size..];
        }
    }
    if region.instruction_pcs.iter().any(|pc| {
        !region
            .source_bytes
            .iter()
            .any(|(address, bytes)| pc >= address && u64::from(*pc) + u64::from(alignment) <= u64::from(*address) + bytes.len() as u64)
    }) {
        return Err(String::from("cached instruction is outside source spans"));
    }
    source.sort_unstable_by_key(|stamp| stamp.page);
    source.dedup_by_key(|stamp| stamp.page);
    region.source = source;
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::sync::atomic::{AtomicUsize, Ordering};

    use wie_arm_jit_types::{
        CodeImage, CodePageStamp, CompileRegion, RegionKey,
        ir::{BasicBlock, Condition, Instruction, Operation, RegionIr},
    };

    use super::*;

    fn region_with_blocks(entry: RegionKey, blocks: &[(u32, usize)]) -> CompileRegion {
        let size = if entry.thumb { 2 } else { 4 };
        // MOV r8,r8 and PLD are instruction-neutral source bytes for these compiler fixtures.
        let encoded: &[u8] = if entry.thumb { &[0xc0, 0x46] } else { &[0x00, 0xf0, 0xd0, 0xf5] };
        let source_bytes: Vec<_> = blocks.iter().map(|(pc, count)| (*pc, encoded.repeat(*count))).collect();
        let mut source: Vec<_> = blocks
            .iter()
            .map(|(pc, _)| CodePageStamp {
                page: pc & !0xffff,
                version: 7,
            })
            .collect();
        source.sort_unstable_by_key(|stamp| stamp.page);
        source.dedup_by_key(|stamp| stamp.page);
        CompileRegion {
            ir: RegionIr {
                entry,
                blocks: blocks
                    .iter()
                    .map(|(pc, count)| BasicBlock {
                        instructions: (0..*count)
                            .map(|index| Instruction {
                                pc: pc + index as u32 * u32::from(size),
                                size,
                                condition: Condition::Always,
                                operation: Operation::Nop,
                            })
                            .collect(),
                    })
                    .collect(),
            },
            source,
            source_bytes,
        }
    }

    #[test]
    fn compiler_coalesces_across_yields_and_flushes_once_at_eof() {
        let entry = RegionKey {
            pc: 0x1004,
            thumb: false,
            cpu_mode: 0x1f,
        };
        let first = region_with_blocks(entry, &[(0x1004, 2)]);
        let second = region_with_blocks(RegionKey { pc: 0x1010, ..entry }, &[(0x1010, 1)]);
        let expected_source_bytes = [first.source_bytes.as_slice(), second.source_bytes.as_slice()].concat();
        let mut image = CodeImage {
            address: 0x1000,
            bytes: vec![0xff; 0x20],
            source: vec![CodePageStamp { page: 0, version: 7 }],
        };
        for (pc, bytes) in &expected_source_bytes {
            let start = (pc - image.address) as usize;
            image.bytes[start..start + bytes.len()].copy_from_slice(bytes);
        }
        let mut inputs = vec![
            Some(first),
            None,
            Some(second),
            Some(region_with_blocks(RegionKey { pc: 0x4000, ..entry }, &[(0x4000, 1)])),
        ]
        .into_iter();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let mut compiler = Compiler::new(CompileRequest {
            images: Arc::from([]),
            regions: Box::new(core::iter::from_fn(move || {
                observed.fetch_add(1, Ordering::Relaxed);
                inputs.next()
            })),
        });
        for (step, emitted) in [(1, 0), (2, 0), (3, 0), (4, 1), (5, 2)] {
            assert!(!compiler.step().unwrap());
            assert_eq!(calls.load(Ordering::Relaxed), step);
            assert_eq!(compiler.artifact.manifest.len(), emitted);
            assert!(compiler.artifact.bytes.is_empty());
        }
        loop {
            let previous_size = compiler.artifact.bytes.len();
            let complete = compiler.step().unwrap();
            assert!(compiler.artifact.bytes.len() - previous_size <= COPY_SIZE);
            assert_eq!(calls.load(Ordering::Relaxed), 5);
            if complete {
                break;
            }
        }
        assert!(compiler.step().unwrap());
        assert_eq!(calls.load(Ordering::Relaxed), 5);
        let artifact = compiler.finish();
        assert_eq!(artifact.manifest.len(), 2);
        let merged = &artifact.manifest[0];
        assert_eq!(merged.entry, entry);
        assert_eq!(merged.instruction_pcs, [0x1004, 0x1008, 0x1010]);
        assert_eq!(merged.source, image.source);
        assert_eq!(merged.source_bytes, expected_source_bytes);
        assert_eq!(artifact.manifest[1].instruction_pcs, [0x4000]);
        let mut restored = merged.clone();
        restored.source.clear();
        image.bytes[0xc] ^= 1; // The gap between source spans is not translated code.
        bind_manifest_source(&mut restored, &[image]).unwrap();
        assert_eq!(restored.source, merged.source);
    }

    #[test]
    fn cached_manifest_rebinds_current_stamps_and_preserves_slot_order() {
        let images = [CodeImage {
            address: 0xfffc,
            bytes: vec![0, 0, 0xa0, 0xe1, 0x1e, 0xff, 0x2f, 0xe1],
            source: vec![CodePageStamp { page: 0, version: 42 }, CodePageStamp { page: 0x10000, version: 43 }],
        }];
        let manifest: Vec<_> = [false, true]
            .into_iter()
            .map(|thumb| ManifestRegion {
                entry: RegionKey {
                    pc: 0xfffc,
                    thumb,
                    cpu_mode: 0x1f,
                },
                instruction_pcs: if thumb {
                    vec![0xfffc, 0xfffe, 0x10000, 0x10002]
                } else {
                    vec![0xfffc, 0x10000]
                },
                source: vec![CodePageStamp { page: 0, version: 7 }, CodePageStamp { page: 0x10000, version: 7 }],
                source_bytes: vec![(0xfffc, images[0].bytes.clone())],
            })
            .collect();
        let mut bytes = Vec::new();
        for region in &manifest {
            encode_manifest_region(region, &mut bytes);
        }
        let mut remaining = bytes.as_slice();
        let mut restored = Vec::new();
        while !remaining.is_empty() {
            let mut region = decode_manifest_region(&mut remaining).unwrap();
            assert!(region.source.is_empty());
            bind_manifest_source(&mut region, &images).unwrap();
            restored.push(region);
        }
        assert_eq!(restored.len(), manifest.len());
        for (actual, expected) in restored.iter().zip(&manifest) {
            assert_eq!(actual.entry, expected.entry);
            assert_eq!(actual.instruction_pcs, expected.instruction_pcs);
            assert_eq!(actual.source_bytes, expected.source_bytes);
            assert_eq!(actual.source, images[0].source);
        }
        restored[0].source_bytes[0].1[0] ^= 1;
        assert!(bind_manifest_source(&mut restored[0], &images).is_err());
    }

    #[test]
    fn cached_source_spans_cross_adjacent_images_and_reach_the_last_guest_byte() {
        let images = [
            CodeImage {
                address: 0xfffc,
                bytes: vec![0, 0, 0xa0, 0xe1],
                source: vec![CodePageStamp { page: 0, version: 11 }],
            },
            CodeImage {
                address: 0x10000,
                bytes: vec![0x1e, 0xff, 0x2f, 0xe1],
                source: vec![CodePageStamp { page: 0x10000, version: 12 }],
            },
            CodeImage {
                address: u32::MAX - 3,
                bytes: vec![0x1e, 0xff, 0x2f, 0xe1],
                source: vec![CodePageStamp {
                    page: 0xffff0000,
                    version: 13,
                }],
            },
        ];
        let mut region = ManifestRegion {
            entry: RegionKey {
                pc: 0xfffc,
                thumb: false,
                cpu_mode: 0x1f,
            },
            instruction_pcs: vec![0xfffc, 0x10000],
            source: Vec::new(),
            source_bytes: vec![(0xfffc, [images[0].bytes.as_slice(), images[1].bytes.as_slice()].concat())],
        };
        bind_manifest_source(&mut region, &images).unwrap();
        assert_eq!(region.source, [images[0].source[0], images[1].source[0]]);
        assert!(bind_manifest_source(&mut region, &images[..1]).is_err());
        region.entry.pc = u32::MAX - 3;
        region.instruction_pcs = vec![region.entry.pc];
        region.source_bytes = vec![(region.entry.pc, images[2].bytes.clone())];
        bind_manifest_source(&mut region, &images).unwrap();
        assert_eq!(region.source, images[2].source);
        region.source_bytes[0].1.extend_from_slice(&[0; 4]);
        assert!(bind_manifest_source(&mut region, &images).is_err());
    }

    #[test]
    fn cached_manifest_rejects_truncated_bytes_source_and_instruction_ownership() {
        let images = [CodeImage {
            address: 0x1000,
            bytes: vec![1, 0x30, 0x70, 0x47],
            source: vec![CodePageStamp { page: 0, version: 7 }],
        }];
        let valid = ManifestRegion {
            entry: RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            },
            instruction_pcs: vec![0x1000, 0x1002],
            source: images[0].source.clone(),
            source_bytes: vec![(0x1000, images[0].bytes.clone())],
        };
        for case in 0..12 {
            let mut region = valid.clone();
            match case {
                0 => region.source_bytes.clear(),
                1 => region.source_bytes[0].1.clear(),
                2 => region.source_bytes[0].0 = 0x2000,
                3 => region.source_bytes[0].1[0] ^= 1,
                4 => region.source_bytes[0].0 = u32::MAX - 1,
                5 => region.instruction_pcs.clear(),
                6 => region.instruction_pcs.push(0x1002),
                7 => region.instruction_pcs.reverse(),
                8 => region.instruction_pcs[1] = 0x1001,
                9 => region.instruction_pcs[1] = 0x1004,
                10 => region.entry.pc = 0x1004,
                11 => region.entry.cpu_mode = 0x10,
                _ => unreachable!(),
            }
            assert!(bind_manifest_source(&mut region, &images).is_err(), "case {case}");
        }
        let mut encoded = Vec::new();
        encode_manifest_region(&valid, &mut encoded);
        for length in 0..encoded.len() {
            assert!(decode_manifest_region(&mut &encoded[..length]).is_none(), "length={length}");
        }
        for offset in [4, 8, 20, 28] {
            let mut corrupt = encoded.clone();
            corrupt[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(decode_manifest_region(&mut corrupt.as_slice()).is_none(), "offset={offset}");
        }
    }
}
