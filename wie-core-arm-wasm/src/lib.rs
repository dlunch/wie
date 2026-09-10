#![no_std]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};

use wie_arm_jit_types::{CompileRequest, ManifestRegion};

mod codegen;

#[cfg(target_arch = "wasm32")]
mod runtime;

#[cfg(target_arch = "wasm32")]
pub use runtime::{WasmExecutor, now};

const COPY_SIZE: usize = 64 * 1024;

pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub manifest: Vec<ManifestRegion>,
}

pub struct Compiler {
    request: CompileRequest,
    builder: Option<codegen::ModuleBuilder>,
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
            let Some(region) = self.request.next() else {
                let builder = core::mem::take(builder);
                self.builder = None;
                self.chunks = builder.begin_assembly(&mut self.artifact.bytes)?;
                return Ok(false);
            };
            if let Some(region) = region {
                let export = builder.add_region(&region.ir);
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
                    export,
                });
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
