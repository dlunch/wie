#![no_std]
extern crate alloc;

use alloc::{boxed::Box, collections::VecDeque, string::String, vec::Vec};

use wie_core_arm_aot::{CompileRegion, CompileRequest, ManifestRegion};

mod codegen;

pub struct WasmArtifact {
    /// Module bytes in order, kept as separate chunks so hosts can release each one as they copy it out.
    pub chunks: VecDeque<Vec<u8>>,
    pub manifest: Vec<ManifestRegion>,
}

pub struct Compiler {
    regions: Box<dyn Iterator<Item = Option<CompileRegion>> + Send>,
    builder: Option<codegen::ModuleBuilder>,
    artifact: WasmArtifact,
}

impl Compiler {
    pub fn new(request: CompileRequest) -> Self {
        Self {
            regions: Box::new(request.coalesced_regions()),
            builder: Some(codegen::ModuleBuilder::default()),
            artifact: WasmArtifact {
                chunks: VecDeque::new(),
                manifest: Vec::new(),
            },
        }
    }

    /// Consumes one bounded decoder step, or assembles the module once every region is emitted.
    pub fn step(&mut self) -> Result<bool, String> {
        let Some(builder) = &mut self.builder else {
            return Ok(true);
        };
        let region = match self.regions.next() {
            Some(None) => return Ok(false),
            region => region.flatten(),
        };
        let Some(region) = region else {
            self.artifact.chunks = self.builder.take().unwrap().assemble()?;
            return Ok(true);
        };
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

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use core::sync::atomic::{AtomicUsize, Ordering};

    use wie_core_arm_aot::{
        RegionKey,
        ir::{BasicBlock, Condition, Instruction, MemoryAddress, Operation, RegionIr},
    };

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
            assert!(compiler.artifact.chunks.is_empty());
        }
        assert!(compiler.step().unwrap());
        assert!(!compiler.artifact.chunks.is_empty());
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
}
