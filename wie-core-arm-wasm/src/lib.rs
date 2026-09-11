#![no_std]
extern crate alloc;

use alloc::{collections::VecDeque, string::String, vec::Vec};

use wie_arm_jit_types::{CodeImage, CompileRequest, ManifestRegion};

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
            let Some(region) = self.request.regions.next() else {
                let builder = core::mem::take(builder);
                self.builder = None;
                self.chunks = builder.begin_assembly(&mut self.artifact.bytes)?;
                return Ok(false);
            };
            if let Some(region) = region {
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
    use alloc::vec;

    use wie_arm_jit_types::{CodeImage, CodePageStamp, RegionKey};

    use super::*;

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
            let value = serde_json::to_value(region).unwrap();
            assert!(value.get("source").is_none());
            bytes.extend(serde_json::to_vec(region).unwrap());
            bytes.push(b'\n');
        }
        let restored: Vec<_> = serde_json::Deserializer::from_slice(&bytes)
            .into_iter::<ManifestRegion>()
            .map(|region| {
                let mut region = region.unwrap();
                assert!(region.source.is_empty());
                bind_manifest_source(&mut region, &images).unwrap();
                region
            })
            .collect();
        assert_eq!(restored.len(), manifest.len());
        for (actual, expected) in restored.iter().zip(&manifest) {
            assert_eq!(actual.entry, expected.entry);
            assert_eq!(actual.instruction_pcs, expected.instruction_pcs);
            assert_eq!(actual.source_bytes, expected.source_bytes);
            assert_eq!(actual.source, images[0].source);
        }
        let mut injected = serde_json::to_value(&manifest[0]).unwrap();
        injected["source"] = serde_json::json!([{"page": 0, "version": 42}, "not a stamp"]);
        let mut restored: ManifestRegion = serde_json::from_value(injected).unwrap();
        assert!(restored.source.is_empty());
        bind_manifest_source(&mut restored, &images).unwrap();
        assert_eq!(restored.source, images[0].source);
        restored.source_bytes[0].1[0] ^= 1;
        assert!(bind_manifest_source(&mut restored, &images).is_err());
        assert!(serde_json::Deserializer::from_slice(b"").into_iter::<ManifestRegion>().next().is_none());
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
    fn cached_manifest_rejects_corrupt_json_source_and_instruction_ownership() {
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
        let encoded = serde_json::to_vec(&valid).unwrap();
        assert!(
            serde_json::Deserializer::from_slice(&encoded[..encoded.len() - 1])
                .into_iter::<ManifestRegion>()
                .next()
                .unwrap()
                .is_err()
        );
        for invalid in [
            serde_json::json!(-1),
            serde_json::json!(4294967296_u64),
            serde_json::json!(1.5),
            serde_json::json!("4096"),
        ] {
            let mut value = serde_json::to_value(&valid).unwrap();
            value["entry"]["pc"] = invalid;
            assert!(serde_json::from_value::<ManifestRegion>(value).is_err());
        }
    }
}
