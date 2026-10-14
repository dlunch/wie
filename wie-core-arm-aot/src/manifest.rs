use alloc::{collections::BTreeSet, string::String, vec::Vec};

use nom::{Parser, multi::length_count, number::complete::le_u32};

use crate::{CodeImage, ManifestRegion, RegionKey};

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

/// Validates cached instruction coverage against input image bounds and ownership across an artifact.
pub fn validate_manifest_region(region: &ManifestRegion, images: &[CodeImage], owned: &mut BTreeSet<RegionKey>) -> Result<(), String> {
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
    for &pc in &region.instruction_pcs {
        if !owned.insert(RegionKey { pc, ..region.entry }) {
            return Err(String::from("duplicate cached instruction ownership"));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

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
        let mut owned = BTreeSet::new();
        for expected in &manifest {
            let actual = decode_manifest_region(&mut remaining).unwrap();
            validate_manifest_region(&actual, &images, &mut owned).unwrap();
            assert!(actual.entry == expected.entry);
            assert_eq!(actual.instruction_pcs, expected.instruction_pcs);
            assert_eq!(actual.code_ranges, expected.code_ranges);
            assert!(validate_manifest_region(&actual, &images[..1], &mut BTreeSet::new()).is_err());
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
            assert!(validate_manifest_region(&region, &images, &mut BTreeSet::new()).is_err(), "case {case}");
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
        let mut owned = BTreeSet::new();
        validate_manifest_region(&valid(), &images, &mut owned).unwrap();
        let mut overlapping = valid();
        overlapping.entry.pc = 0x1002;
        overlapping.instruction_pcs.remove(0);
        overlapping.code_ranges[0].start = 0x1002;
        assert_eq!(
            validate_manifest_region(&overlapping, &images, &mut owned).unwrap_err(),
            "duplicate cached instruction ownership"
        );
    }
}
