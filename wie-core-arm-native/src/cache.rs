use alloc::{collections::BTreeSet, format, vec::Vec};
use core::mem::{align_of, offset_of, size_of};

use cranelift_codegen::isa::TargetIsa;
use nom::{Parser, bytes::complete::take, number::complete::le_u32};
use sha2::{Digest, Sha256};

use wie_arm_jit_types::{
    CodeImage, CompileRequest, ManifestRegion, MemoryPage, RunFrame,
    manifest::{decode_manifest_region, encode_manifest_region, validate_manifest_region},
};
use wie_util::Result;

use crate::PendingRegion;

const MAGIC: &[u8; 8] = b"WIEAOT\0\0";
/// Increment when decoding, coalescing, generated code, its ABI, or this format changes.
const VERSION: u32 = 1;
const DIGEST_OFFSET: usize = 8 + 4 + 32;

/// Host storage for this application's private native executable cache.
///
/// # Safety
/// Loaded records must originate from this compiler/cache pipeline. Storage must
/// remain outside guest, import, download, and other untrusted write paths.
/// Checksums detect corruption; they do not authenticate executable code.
pub unsafe trait NativeCache: Send {
    fn load(&mut self, key: &[u8; 32]) -> Result<Option<Vec<u8>>>;

    /// # Safety
    /// `artifact` must be the complete output of this compiler/cache pipeline for `key`.
    unsafe fn store(&mut self, key: &[u8; 32], artifact: &[u8]) -> Result<()>;
}

pub(crate) fn key(isa: &dyn TargetIsa, request: &CompileRequest) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(MAGIC);
    hash.update(VERSION.to_le_bytes());
    for field in [
        cranelift_codegen::VERSION.as_bytes(),
        cranelift_module::VERSION.as_bytes(),
        cranelift_jit::VERSION.as_bytes(),
        format!(
            "{}:{}:{:?}:{}",
            isa.triple(),
            isa.default_call_conv(),
            isa.endianness(),
            isa.pointer_bits()
        )
        .as_bytes(),
    ] {
        hash.update((field.len() as u64).to_le_bytes());
        hash.update(field);
    }
    for mut flags in [isa.flags().iter().collect::<Vec<_>>(), isa.isa_flags()] {
        flags.sort_unstable_by_key(|flag| flag.name);
        hash.update((flags.len() as u64).to_le_bytes());
        for flag in flags {
            let field = format!("{flag}");
            hash.update((field.len() as u64).to_le_bytes());
            hash.update(field.as_bytes());
        }
    }
    for field in [
        size_of::<RunFrame>(),
        align_of::<RunFrame>(),
        offset_of!(RunFrame, regs),
        offset_of!(RunFrame, cpsr),
        offset_of!(RunFrame, fault_address),
        size_of::<MemoryPage>(),
        align_of::<MemoryPage>(),
        offset_of!(MemoryPage, bytes),
        request.max_region_instructions,
        request.max_region_blocks,
        request.images.len(),
    ] {
        hash.update((field as u64).to_le_bytes());
    }
    for image in &*request.images {
        hash.update(image.address.to_le_bytes());
        hash.update((image.bytes.len() as u64).to_le_bytes());
        hash.update(&image.bytes);
    }
    hash.finalize().into()
}

pub(crate) fn encode<'a>(key: &[u8; 32], regions: impl Iterator<Item = &'a PendingRegion>) -> Option<Vec<u8>> {
    let mut regions: Vec<_> = regions.collect();
    regions.sort_unstable_by_key(|region| region.index);
    let count = u32::try_from(regions.len()).ok()?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(MAGIC);
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(key);
    bytes.extend_from_slice(&[0; 32]);
    bytes.extend_from_slice(&count.to_le_bytes());
    for region in regions {
        let (alignment, code) = region.code.as_ref()?;
        let mut manifest = Vec::new();
        encode_manifest_region(&region.manifest, &mut manifest);
        bytes.extend_from_slice(&alignment.to_le_bytes());
        bytes.extend_from_slice(&u32::try_from(code.len()).ok()?.to_le_bytes());
        bytes.extend_from_slice(&u32::try_from(manifest.len()).ok()?.to_le_bytes());
        bytes.extend_from_slice(code);
        bytes.extend_from_slice(&manifest);
    }
    let mut hash = Sha256::new();
    hash.update(&bytes[..DIGEST_OFFSET]);
    hash.update(&bytes[DIGEST_OFFSET + 32..]);
    bytes[DIGEST_OFFSET..DIGEST_OFFSET + 32].copy_from_slice(&hash.finalize());
    Some(bytes)
}

pub(crate) struct CachedRegion<'a> {
    pub alignment: u32,
    pub code: &'a [u8],
    pub manifest: ManifestRegion,
}

pub(crate) fn decode<'a>(bytes: &'a [u8], key: &[u8; 32], images: &[CodeImage]) -> Option<Vec<CachedRegion<'a>>> {
    let header: nom::IResult<_, _> = (take(8usize), le_u32, take(32usize), take(32usize), le_u32).parse(bytes);
    let (mut input, (magic, version, stored_key, digest, count)) = header.ok()?;
    let mut hash = Sha256::new();
    hash.update(&bytes[..DIGEST_OFFSET]);
    hash.update(&bytes[DIGEST_OFFSET + 32..]);
    if magic != MAGIC || version != VERSION || stored_key != key || digest != &hash.finalize()[..] {
        return None;
    }
    let mut regions = Vec::new();
    let mut owned = BTreeSet::new();
    for _ in 0..count {
        let lengths: nom::IResult<_, _> = (le_u32, le_u32, le_u32).parse(input);
        let (remaining, (alignment, code_length, manifest_length)) = lengths.ok()?;
        if !alignment.is_power_of_two() || code_length == 0 {
            return None;
        }
        let record: nom::IResult<_, _> = (take(code_length as usize), take(manifest_length as usize)).parse(remaining);
        let (remaining, (code, mut manifest)) = record.ok()?;
        let region = decode_manifest_region(&mut manifest)?;
        if !manifest.is_empty() {
            return None;
        }
        validate_manifest_region(&region, images, &mut owned).ok()?;
        regions.push(CachedRegion {
            alignment,
            code,
            manifest: region,
        });
        input = remaining;
    }
    input.is_empty().then_some(regions)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};

    use cranelift_codegen::{isa, settings, settings::Configurable};
    use cranelift_module::{FuncId, Module};
    use wie_arm_jit_types::RegionKey;

    use super::*;

    #[test]
    fn identity_covers_images_compilation_limits_and_effective_isa() {
        let input = || CompileRequest {
            images: Arc::from([
                CodeImage {
                    address: 0x1000,
                    bytes: vec![1, 2, 3, 4],
                },
                CodeImage {
                    address: 0x2000,
                    bytes: vec![5, 6, 7, 8],
                },
            ]),
            max_region_instructions: 4096,
            max_region_blocks: 512,
            regions: Box::new(core::iter::empty()),
        };
        let module = crate::create_module().unwrap();
        let isa = module.0.as_ref().unwrap().isa();
        let original = key(isa, &input());
        assert_eq!(original, key(isa, &input()));
        for change in 0..6 {
            let mut request = input();
            match change {
                0 => Arc::get_mut(&mut request.images).unwrap()[0].address += 4,
                1 => Arc::get_mut(&mut request.images).unwrap()[0].bytes[0] ^= 1,
                2 => Arc::get_mut(&mut request.images).unwrap()[0].bytes.push(0),
                3 => Arc::get_mut(&mut request.images).unwrap().swap(0, 1),
                4 => request.max_region_instructions += 1,
                5 => request.max_region_blocks += 1,
                _ => unreachable!(),
            }
            assert_ne!(original, key(isa, &request), "change {change}");
        }
        let mut identities = BTreeSet::new();
        for triple in ["x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc", "aarch64-unknown-linux-gnu"] {
            for speed in [false, true] {
                let mut flags = settings::builder();
                flags.set("opt_level", if speed { "speed" } else { "none" }).unwrap();
                let isa = isa::lookup(triple.parse().unwrap()).unwrap().finish(settings::Flags::new(flags)).unwrap();
                assert!(identities.insert(key(isa.as_ref(), &input())));
            }
        }
    }

    #[test]
    fn artifact_rejects_corruption_and_invalid_records_before_loading_code() {
        let key = [7; 32];
        let images = [CodeImage {
            address: 0x1000,
            bytes: vec![0; 4],
        }];
        let region = || PendingRegion {
            index: 0,
            function: FuncId::from_u32(0),
            manifest: ManifestRegion {
                entry: RegionKey {
                    pc: 0x1000,
                    thumb: false,
                    cpu_mode: 0x1f,
                },
                instruction_pcs: vec![0x1000],
                code_ranges: core::iter::once(0x1000..0x1004).collect(),
            },
            // The codec test never allocates or executes these bytes.
            code: Some((1, vec![1])),
        };
        let encoded = encode(&key, [&region()].into_iter()).unwrap();
        let decoded = decode(&encoded, &key, &images).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].code, [1]);
        assert_eq!(decoded[0].manifest.instruction_pcs, [0x1000]);
        for length in 0..encoded.len() {
            assert!(decode(&encoded[..length], &key, &images).is_none(), "truncated at {length}");
        }
        for offset in [0, 8, 12, DIGEST_OFFSET, DIGEST_OFFSET + 32, 92] {
            let mut corrupt = encoded.clone();
            corrupt[offset] ^= 1;
            assert!(decode(&corrupt, &key, &images).is_none(), "corruption at {offset}");
        }
        assert!(decode(&encoded, &[8; 32], &images).is_none());

        let record = DIGEST_OFFSET + 32 + 4;
        let manifest = record + 12 + 1;
        for (offset, value) in [
            (8, VERSION + 1),
            (record - 4, 0),
            (record - 4, u32::MAX),
            (record, 3),
            (record + 4, 0),
            (record + 4, u32::MAX),
            (record + 8, u32::MAX),
            (manifest + 4, 0x10 << 1),
            (manifest + 12, 0x2000),
        ] {
            let mut corrupt = encoded.clone();
            corrupt[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            let mut hash = Sha256::new();
            hash.update(&corrupt[..DIGEST_OFFSET]);
            hash.update(&corrupt[DIGEST_OFFSET + 32..]);
            corrupt[DIGEST_OFFSET..DIGEST_OFFSET + 32].copy_from_slice(&hash.finalize());
            assert!(decode(&corrupt, &key, &images).is_none(), "invalid metadata at {offset}: {value}");
        }
        let duplicate = encode(&key, [&region(), &region()].into_iter()).unwrap();
        assert!(decode(&duplicate, &key, &images).is_none());
        let empty = encode(&key, core::iter::empty()).unwrap();
        assert!(decode(&empty, &key, &images).unwrap().is_empty());
    }
}
