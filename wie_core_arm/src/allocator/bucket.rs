use alloc::vec;

use wie_util::{ByteRead, ByteWrite, Result, WieError};

use crate::core::ArmCore;

pub const BUCKET_MAX: usize = 512;

// (slot_size, slot_count). slot_count is a multiple of 8 so the header has no
// trailing partial byte to mask. Sizes chosen generously per slot class to fit
// inside the 128 MB BucketAllocator half of the heap; total layout is
// ~0x785A000 (~120 MB), leaving ~8 MB of intentional slack.
const BUCKETS: [(usize, usize); 8] = [
    (4, 0x100000),
    (8, 0x80000),
    (16, 0x80000),
    (32, 0x40000),
    (64, 0x40000),
    (128, 0x20000),
    (256, 0x20000),
    (512, 0x10000),
];

const fn header_length(idx: usize) -> usize {
    BUCKETS[idx].1 / 8
}

const fn region_size(idx: usize) -> usize {
    let (slot_size, slot_count) = BUCKETS[idx];
    slot_count / 8 + slot_size * slot_count
}

const fn region_offset(idx: usize) -> usize {
    let mut offset = 0;
    let mut i = 0;
    while i < idx {
        offset += region_size(i);
        i += 1;
    }
    offset
}

const fn total_size() -> usize {
    region_offset(BUCKETS.len() - 1) + region_size(BUCKETS.len() - 1)
}

pub struct BucketAllocator;

impl BucketAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, base_size: u32) -> Result<()> {
        // header contains bitset of allocation, 1 is unallocated, 0 is allocated

        let required = total_size() as u32;
        if base_size < required {
            tracing::error!("BucketAllocator region too small: got {base_size:#x}, need {required:#x}");
            return Err(WieError::AllocationFailure);
        }

        for (i, &(slot_size, _)) in BUCKETS.iter().enumerate() {
            let header_address = base_address + region_offset(i) as u32;
            let header_len = header_length(i);

            tracing::info!("Bucket {slot_size} header size {header_len}");

            core.write_bytes(header_address, &vec![0xff; header_len])?;
        }

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, base_address: u32, size: u32) -> Result<u32> {
        let bucket_index = Self::find_bucket_index(size);
        let (slot_size, _) = BUCKETS[bucket_index];
        let header_address = base_address + region_offset(bucket_index) as u32;
        let header_len = header_length(bucket_index);

        let mut header = vec![0u8; header_len];
        core.read_bytes(header_address, &mut header)?;

        for (i, item) in header.iter_mut().enumerate() {
            if *item == 0 {
                continue;
            }

            let bit = item.trailing_zeros();
            *item &= !(1 << bit);
            let address = header_address + header_len as u32 + (i as u32 * 8 + bit) * slot_size as u32;

            core.write_bytes(header_address + i as u32, &[*item])?;

            return Ok(address);
        }

        Err(WieError::AllocationFailure)
    }

    pub fn free(core: &mut ArmCore, base_address: u32, address: u32, size: u32) -> Result<()> {
        let bucket_index = Self::find_bucket_index(size);
        let (slot_size, _) = BUCKETS[bucket_index];
        let header_address = base_address + region_offset(bucket_index) as u32;
        let header_len = header_length(bucket_index);

        let mut header = vec![0u8; header_len];
        core.read_bytes(header_address, &mut header)?;

        let offset = (address - header_address - header_len as u32) / slot_size as u32;
        let index = offset / 8;
        let bit = offset % 8;

        debug_assert!(header[index as usize] & (1 << bit) == 0);

        header[index as usize] |= 1 << bit;

        core.write_bytes(header_address + index, &[header[index as usize]])?;

        Ok(())
    }

    fn find_bucket_index(size: u32) -> usize {
        BUCKETS.iter().position(|&(s, _)| size as usize <= s).unwrap_or(BUCKETS.len() - 1)
    }
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use crate::ArmCore;

    use super::BucketAllocator;

    // Bucket 0 (4-byte): header_length = 0x100000 / 8 = 0x20000.
    //   First slot at base + 0x20000 = 0x40020000.
    // Bucket 1 (8-byte): region_offset = 0x20000 + 4*0x100000 = 0x420000.
    //   header_length = 0x80000 / 8 = 0x10000.
    //   First slot at base + 0x420000 + 0x10000 = 0x40430000.

    #[test]
    fn test_allocator() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x8000000)?;

        BucketAllocator::init(&mut core, 0x40000000, 0x8000000)?;

        let address1 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address1, 0x40020000);

        let address2 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address2, 0x40020004);

        let address3 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address3, 0x40020008);

        BucketAllocator::free(&mut core, 0x40000000, address2, 4)?;

        let address4 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address4, 0x40020004);

        let address5 = BucketAllocator::alloc(&mut core, 0x40000000, 5)?;
        assert_eq!(address5, 0x40430000);

        let address6 = BucketAllocator::alloc(&mut core, 0x40000000, 6)?;
        assert_eq!(address6, 0x40430008);

        let address7 = BucketAllocator::alloc(&mut core, 0x40000000, 7)?;
        assert_eq!(address7, 0x40430010);

        let address8 = BucketAllocator::alloc(&mut core, 0x40000000, 8)?;
        assert_eq!(address8, 0x40430018);

        Ok(())
    }

    #[test]
    fn test_init_rejects_undersized_region() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000000).unwrap();

        // 0x1000000 (16 MB) is far too small for the full bucket layout.
        assert!(BucketAllocator::init(&mut core, 0x40000000, 0x1000000).is_err());
    }

    #[test]
    fn test_allocator_small_sizes() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x8000000)?;

        BucketAllocator::init(&mut core, 0x40000000, 0x8000000)?;

        // sizes 0..=3 must all land in the smallest (4-byte) bucket
        let a0 = BucketAllocator::alloc(&mut core, 0x40000000, 0)?;
        assert_eq!(a0, 0x40020000);
        let a1 = BucketAllocator::alloc(&mut core, 0x40000000, 1)?;
        assert_eq!(a1, 0x40020004);
        let a2 = BucketAllocator::alloc(&mut core, 0x40000000, 2)?;
        assert_eq!(a2, 0x40020008);
        let a3 = BucketAllocator::alloc(&mut core, 0x40000000, 3)?;
        assert_eq!(a3, 0x4002000c);

        // free with the same (small) size must round-trip without panicking
        BucketAllocator::free(&mut core, 0x40000000, a1, 1)?;
        let a1b = BucketAllocator::alloc(&mut core, 0x40000000, 1)?;
        assert_eq!(a1b, 0x40020004);

        Ok(())
    }
}
