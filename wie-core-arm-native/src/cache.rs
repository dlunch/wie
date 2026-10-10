use alloc::{collections::BTreeSet, format, vec::Vec};
use core::mem::{align_of, offset_of, size_of};

use cranelift_codegen::isa::TargetIsa;
use nom::{Parser, bytes::complete::take, number::complete::le_u32};
use sha2::{Digest, Sha256};

use wie_arm_aot::{
    CodeImage, CompileRequest, ManifestRegion, MemoryPage, RunFrame, cache as artifact,
    manifest::{decode_manifest_region, encode_manifest_region, validate_manifest_region},
};
use wie_util::Result;

use crate::PendingRegion;

/// Host storage for this application's private native executable cache.
pub trait NativeCache: Send {
    fn load(&mut self, key: &[u8; 32]) -> Result<Option<Vec<u8>>>;
    fn store(&mut self, key: &[u8; 32], artifact: &[u8]) -> Result<()>;
}

pub(crate) fn key(isa: &dyn TargetIsa, request: &CompileRequest) -> [u8; 32] {
    let mut hash = Sha256::new();
    hash.update(artifact::key_input(request));
    let target = format!(
        "{}:{}:{:?}:{}",
        isa.triple(),
        isa.default_call_conv(),
        isa.endianness(),
        isa.pointer_bits()
    );
    hash.update((target.len() as u64).to_le_bytes());
    hash.update(target.as_bytes());
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
    ] {
        hash.update((field as u64).to_le_bytes());
    }
    hash.finalize().into()
}

pub(crate) fn encode<'a>(key: &[u8; 32], regions: impl Iterator<Item = &'a PendingRegion>) -> Option<Vec<u8>> {
    let mut regions: Vec<_> = regions.collect();
    regions.sort_unstable_by_key(|region| region.index);
    let mut code = Vec::new();
    let mut manifest = Vec::new();
    for region in regions {
        let (alignment, bytes) = region.code.as_ref()?;
        code.extend_from_slice(&alignment.to_le_bytes());
        code.extend_from_slice(&u32::try_from(bytes.len()).ok()?.to_le_bytes());
        code.extend_from_slice(bytes);
        encode_manifest_region(&region.manifest, &mut manifest);
    }
    let mut bytes = artifact::encode(key, &code, &manifest)?;
    let digest = Sha256::digest(&bytes);
    bytes.extend_from_slice(&digest);
    Some(bytes)
}

pub(crate) struct CachedRegion<'a> {
    pub alignment: u32,
    pub code: &'a [u8],
    pub manifest: ManifestRegion,
}

pub(crate) fn decode<'a>(bytes: &'a [u8], key: &[u8; 32], images: &[CodeImage]) -> Option<Vec<CachedRegion<'a>>> {
    let (payload, digest) = bytes.split_at_checked(bytes.len().checked_sub(32)?)?;
    if digest != &Sha256::digest(payload)[..] {
        return None;
    }
    let (mut input, mut manifest) = artifact::decode(payload, key)?;
    let mut regions = Vec::new();
    let mut owned = BTreeSet::new();
    while !manifest.is_empty() {
        let lengths: nom::IResult<_, _> = (le_u32, le_u32).parse(input);
        let (remaining, (alignment, code_length)) = lengths.ok()?;
        if !alignment.is_power_of_two() || code_length == 0 {
            return None;
        }
        let record: nom::IResult<_, _> = take(code_length as usize).parse(remaining);
        let (remaining, code) = record.ok()?;
        let region = decode_manifest_region(&mut manifest)?;
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
    use wie_arm_aot::RegionKey;

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
    fn artifact_rejects_invalid_native_code_and_manifests_before_loading() {
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
        for offset in 0..encoded.len() {
            let mut corrupt = encoded.clone();
            corrupt[offset] ^= 1;
            assert!(decode(&corrupt, &key, &images).is_none());
        }
        let (code, manifest) = artifact::decode(&encoded[..encoded.len() - 32], &key).unwrap();
        for (offset, value) in [(0, 3u32), (4, 0), (4, u32::MAX)] {
            let mut code = code.to_vec();
            code[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            let mut corrupt = artifact::encode(&key, &code, manifest).unwrap();
            let digest = Sha256::digest(&corrupt);
            corrupt.extend_from_slice(&digest);
            assert!(decode(&corrupt, &key, &images).is_none());
        }
        for (offset, value) in [(4, 0x10u32 << 1), (12, 0x2000)] {
            let mut manifest = manifest.to_vec();
            manifest[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
            let mut corrupt = artifact::encode(&key, code, &manifest).unwrap();
            let digest = Sha256::digest(&corrupt);
            corrupt.extend_from_slice(&digest);
            assert!(decode(&corrupt, &key, &images).is_none());
        }
        for (code, manifest) in [(&[][..], manifest), (code, &[][..])] {
            let mut corrupt = artifact::encode(&key, code, manifest).unwrap();
            let digest = Sha256::digest(&corrupt);
            corrupt.extend_from_slice(&digest);
            assert!(decode(&corrupt, &key, &images).is_none());
        }
        let duplicate = encode(&key, [&region(), &region()].into_iter()).unwrap();
        assert!(decode(&duplicate, &key, &images).is_none());
        let empty = encode(&key, core::iter::empty()).unwrap();
        assert!(decode(&empty, &key, &images).unwrap().is_empty());
    }
}
