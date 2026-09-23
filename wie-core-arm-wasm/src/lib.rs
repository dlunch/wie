#![no_std]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};

use nom::{Parser, multi::length_count, number::complete::le_u32};

use wie_arm_jit_types::{CodeImage, CompileRegion, CompileRequest, ManifestRegion, RegionKey};

mod codegen;

const COPY_SIZE: usize = 64 * 1024;

/// Increment when analysis, generated code, execution ABI, or the cache format changes.
pub const AOT_CACHE_VERSION: u32 = 1;

pub fn encode_manifest_region(region: &ManifestRegion, output: &mut Vec<u8>) {
    // Little-endian header, instruction PCs, then address/length pairs for code coverage.
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
    output.extend_from_slice(&(region.code_ranges.len() as u32).to_le_bytes());
    for range in &region.code_ranges {
        output.extend_from_slice(&(range.start as u32).to_le_bytes());
        output.extend_from_slice(&((range.end - range.start) as u32).to_le_bytes());
    }
}

pub fn decode_manifest_region(input: &mut &[u8]) -> Option<ManifestRegion> {
    let decoded: nom::IResult<_, _> = (le_u32, le_u32, length_count(le_u32, le_u32), length_count(le_u32, (le_u32, le_u32))).parse(*input);
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
        code_ranges: spans
            .into_iter()
            .map(|(address, size)| u64::from(address)..u64::from(address) + u64::from(size))
            .collect(),
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
                            (count + 1, first.min(instruction.pc.get()), last.max(instruction.pc.get()))
                        });
                    // Coalesce small fragments without expanding the selectors of larger regions.
                    let page = (instructions <= 16 && first >> 14 == last >> 14).then_some(first >> 14);
                    if let Some(pending) = &mut self.pending
                        && page.is_some()
                        && page == self.pending_page
                        && pending.ir.entry.thumb == region.ir.entry.thumb
                        && pending.ir.entry.cpu_mode == region.ir.entry.cpu_mode
                        && self.pending_instructions + instructions <= self.request.max_region_instructions
                        && pending.ir.blocks.len() + region.ir.blocks.len() <= self.request.max_region_blocks
                    {
                        self.pending_instructions += instructions;
                        pending.ir.blocks.append(&mut region.ir.blocks);
                        return Ok(false);
                    }
                    self.pending_instructions = instructions;
                    self.pending_page = page;
                    self.pending.replace(region)
                }
                None => self.pending.take(),
            };
            if let Some(region) = region {
                builder.add_region(&region.ir);
                let mut instruction_pcs: Vec<_> = region
                    .ir
                    .blocks
                    .iter()
                    .flat_map(|block| &block.instructions)
                    .map(|instruction| instruction.pc.get())
                    .collect();
                instruction_pcs.sort_unstable();
                self.artifact.manifest.push(ManifestRegion {
                    entry: region.ir.entry,
                    instruction_pcs,
                    code_ranges: region
                        .ir
                        .blocks
                        .iter()
                        .map(|block| {
                            let last = block.instructions.last().unwrap();
                            u64::from(block.instructions[0].pc.get())..u64::from(last.pc.get()) + u64::from(last.size)
                        })
                        .collect(),
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

/// Validates cached instruction coverage against the input image bounds.
pub fn validate_manifest_region(region: &ManifestRegion, images: &[CodeImage]) -> Result<(), String> {
    let alignment = if region.entry.thumb { 2 } else { 4 };
    if region.entry.cpu_mode != 0x1f
        || region.instruction_pcs.binary_search(&region.entry.pc).is_err()
        || region.instruction_pcs.windows(2).any(|pcs| pcs[0] >= pcs[1])
        || region.instruction_pcs.iter().any(|pc| !pc.is_multiple_of(alignment))
        || region.code_ranges.is_empty()
    {
        return Err(String::from("invalid cached instruction coverage"));
    }
    for range in &region.code_ranges {
        if range.is_empty()
            || range.end > u64::from(u32::MAX) + 1
            || !range.start.is_multiple_of(u64::from(alignment))
            || !range.end.is_multiple_of(u64::from(alignment))
            || region.instruction_pcs.binary_search(&(range.start as u32)).is_err()
        {
            return Err(String::from("invalid cached source span"));
        }
        let mut address = range.start;
        while address < range.end {
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
            address += (range.end - address).min(input.len() as u64);
        }
    }
    if region.instruction_pcs.iter().any(|pc| {
        !region
            .code_ranges
            .iter()
            .any(|range| u64::from(*pc) >= range.start && u64::from(*pc) + u64::from(alignment) <= range.end)
    }) {
        return Err(String::from("cached instruction is outside source spans"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::sync::atomic::{AtomicUsize, Ordering};

    use wie_arm_jit_types::ir::{BasicBlock, Condition, Instruction, MemoryAddress, Operation, RegionIr};

    use super::*;

    fn region_with_blocks(entry: RegionKey, blocks: &[(u32, usize)]) -> CompileRegion {
        let size = if entry.thumb { 2 } else { 4 };
        CompileRegion {
            ir: RegionIr {
                entry,
                blocks: blocks
                    .iter()
                    .map(|(pc, count)| BasicBlock {
                        instructions: (0..*count)
                            .map(|index| Instruction {
                                pc: MemoryAddress::new(pc + index as u32 * u32::from(size)),
                                size,
                                condition: Condition::Always,
                                operation: Operation::Nop,
                            })
                            .collect(),
                    })
                    .collect(),
            },
        }
    }

    #[test]
    fn compiler_coalesces_across_yields_and_flushes_once_at_eof() {
        let entry = RegionKey {
            pc: 0x1004,
            thumb: false,
            cpu_mode: 0x1f,
        };
        let mut inputs = vec![
            Some(region_with_blocks(RegionKey { ..entry }, &[(0x1004, 2)])),
            None,
            Some(region_with_blocks(RegionKey { pc: 0x1010, ..entry }, &[(0x1010, 1)])),
            Some(region_with_blocks(RegionKey { pc: 0x4000, ..entry }, &[(0x4000, 1)])),
        ]
        .into_iter();
        let calls = Arc::new(AtomicUsize::new(0));
        let observed = calls.clone();
        let mut compiler = Compiler::new(CompileRequest {
            images: Arc::from([]),
            max_region_instructions: 512,
            max_region_blocks: 128,
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
        assert!(merged.entry == entry);
        assert_eq!(merged.instruction_pcs, [0x1004, 0x1008, 0x1010]);
        assert_eq!(merged.code_ranges, [0x1004..0x100c, 0x1010..0x1014]);
        assert_eq!(artifact.manifest[1].instruction_pcs, [0x4000]);
    }

    #[test]
    fn cached_manifest_preserves_ranges_and_slot_order() {
        let images = [
            CodeImage {
                address: 0xfffc,
                bytes: vec![0; 4],
            },
            CodeImage {
                address: 0x10000,
                bytes: vec![0; 4],
            },
            CodeImage {
                address: u32::MAX - 3,
                bytes: vec![0; 4],
            },
        ];
        let manifest: Vec<_> = [false, true]
            .into_iter()
            .map(|thumb| ManifestRegion {
                entry: RegionKey {
                    pc: 0xfffc,
                    thumb,
                    cpu_mode: 0x1f,
                },
                instruction_pcs: if thumb {
                    vec![0xfffc, 0xfffe, 0x10000, 0x10002, u32::MAX - 3, u32::MAX - 1]
                } else {
                    vec![0xfffc, 0x10000, u32::MAX - 3]
                },
                code_ranges: vec![0xfffc..0x10004, 0xffff_fffc..0x1_0000_0000],
            })
            .collect();
        let mut bytes = Vec::new();
        for region in &manifest {
            encode_manifest_region(region, &mut bytes);
        }
        let mut remaining = bytes.as_slice();
        for expected in &manifest {
            let actual = decode_manifest_region(&mut remaining).unwrap();
            validate_manifest_region(&actual, &images).unwrap();
            assert!(actual.entry == expected.entry);
            assert_eq!(actual.instruction_pcs, expected.instruction_pcs);
            assert_eq!(actual.code_ranges, expected.code_ranges);
            assert!(validate_manifest_region(&actual, &images[..1]).is_err());
        }
        assert!(remaining.is_empty());
    }

    #[test]
    fn cached_manifest_rejects_truncated_data_and_invalid_coverage() {
        let images = [CodeImage {
            address: 0x1000,
            bytes: vec![1, 0x30, 0x70, 0x47],
        }];
        let valid = || ManifestRegion {
            entry: RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            },
            instruction_pcs: vec![0x1000, 0x1002],
            code_ranges: core::iter::once(0x1000..0x1004).collect(),
        };
        for case in 0..12 {
            let mut region = valid();
            match case {
                0 => region.code_ranges.clear(),
                1 => region.code_ranges[0].end = 0x1000,
                2 => region.code_ranges[0] = 0x2000..0x2004,
                3 => region.code_ranges[0].end = 0x1003,
                4 => region.code_ranges[0] = 0xffff_fffe..0x1_0000_0002,
                5 => region.instruction_pcs.clear(),
                6 => region.instruction_pcs.push(0x1002),
                7 => region.instruction_pcs.reverse(),
                8 => region.instruction_pcs[1] = 0x1001,
                9 => region.instruction_pcs[1] = 0x1004,
                10 => region.entry.pc = 0x1004,
                11 => region.entry.cpu_mode = 0x10,
                _ => unreachable!(),
            }
            assert!(validate_manifest_region(&region, &images).is_err(), "case {case}");
        }
        let mut encoded = Vec::new();
        encode_manifest_region(&valid(), &mut encoded);
        for length in 0..encoded.len() {
            assert!(decode_manifest_region(&mut &encoded[..length]).is_none(), "length={length}");
        }
        for offset in [4, 8, 20] {
            let mut corrupt = encoded.clone();
            corrupt[offset..offset + 4].copy_from_slice(&u32::MAX.to_le_bytes());
            assert!(decode_manifest_region(&mut corrupt.as_slice()).is_none(), "offset={offset}");
        }
    }
}
