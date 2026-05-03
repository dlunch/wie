use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, write_generic};

use crate::core::ArmCore;

pub const BUCKET_MAX: usize = 512;
const BUCKETS: [usize; 8] = [4, 8, 16, 32, 64, 128, 256, 512];
// Each bucket gets one BUCKET_SIZE region; 8 buckets * 0x1000000 = 0x8000000
// (128 MB), exactly matching the BucketAllocator half of the heap.
const BUCKET_SIZE: usize = 0x1000000;

// Chunk size for header scans in alloc(). 1 KB balances per-call read overhead
// against unnecessary work for large headers (4-byte bucket header is 512 KB).
const SCAN_CHUNK: usize = 1024;

pub struct BucketAllocator;

impl BucketAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, base_size: u32) -> Result<()> {
        // header contains bitset of allocation, 1 is unallocated, 0 is allocated.
        //
        // Slots beyond `(BUCKET_SIZE - header_length) / bucket` would extend past
        // this bucket's region into the next bucket's slot area; mark those bits
        // as allocated so they're never handed out (otherwise two different
        // bucket sizes can hand out overlapping addresses).
        //
        // The caller-provided region must hold all `BUCKETS.len()` regions of
        // `BUCKET_SIZE` each. Validate up-front so a future heap-sizing change
        // can't silently produce out-of-region writes.
        let required = (BUCKETS.len() * BUCKET_SIZE) as u32;
        if base_size < required {
            tracing::error!("BucketAllocator region too small: got {base_size:#x}, need {required:#x}");
            return Err(WieError::AllocationFailure);
        }

        for (i, bucket) in BUCKETS.into_iter().enumerate() {
            let header_length = BUCKET_SIZE / bucket / 8;
            let header_address = base_address + i as u32 * BUCKET_SIZE as u32;

            let usable_slots = (BUCKET_SIZE - header_length) / bucket;
            let full_bytes = usable_slots / 8;
            let remaining_bits = usable_slots % 8;

            tracing::info!("Bucket {bucket} header size {header_length} usable_slots {usable_slots}");

            // Write the header in chunks so we don't materialize a huge Vec in
            // the smaller buckets (4-byte bucket header is 512 KB).
            let mut chunk = [0u8; SCAN_CHUNK];
            let mut written = 0usize;
            while written < header_length {
                let remaining = header_length - written;
                let take = remaining.min(SCAN_CHUNK);

                for (j, slot) in chunk.iter_mut().take(take).enumerate() {
                    let byte_idx = written + j;
                    *slot = if byte_idx < full_bytes {
                        0xff
                    } else if byte_idx == full_bytes && remaining_bits > 0 {
                        (1u8 << remaining_bits) - 1
                    } else {
                        0
                    };
                }

                core.write_bytes(header_address + written as u32, &chunk[..take])?;
                written += take;
            }
        }

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, base_address: u32, size: u32) -> Result<u32> {
        let bucket_index = Self::find_bucket_index(size);
        let bucket = BUCKETS[bucket_index];
        let base_address = base_address + (bucket_index * BUCKET_SIZE) as u32;
        let header_length = BUCKET_SIZE / bucket / 8;

        // Scan the header in chunks instead of reading the whole thing on every
        // alloc — small buckets have multi-hundred-KB headers and most allocs
        // find a free bit in the first chunk anyway.
        let mut chunk = [0u8; SCAN_CHUNK];
        let mut scanned = 0usize;
        while scanned < header_length {
            let remaining = header_length - scanned;
            let take = remaining.min(SCAN_CHUNK);
            core.read_bytes(base_address + scanned as u32, &mut chunk[..take])?;

            for (j, item) in chunk.iter_mut().take(take).enumerate() {
                if *item == 0 {
                    continue;
                }

                let bit = item.trailing_zeros();
                *item &= !(1 << bit);
                let byte_idx = scanned + j;
                let address = base_address + (header_length as u32) + (byte_idx as u32 * 8 + bit) * bucket as u32;

                core.write_bytes(base_address + byte_idx as u32, &[*item])?;

                return Ok(address);
            }

            scanned += take;
        }

        Err(WieError::AllocationFailure)
    }

    pub fn free(core: &mut ArmCore, base_address: u32, address: u32, size: u32) -> Result<()> {
        let bucket_index = Self::find_bucket_index(size);
        let bucket = BUCKETS[bucket_index];
        let base_address = base_address + (bucket_index * BUCKET_SIZE) as u32;
        let header_length = BUCKET_SIZE / bucket / 8;

        let offset = (address - base_address - header_length as u32) / bucket as u32;
        let index = offset / 8;
        let bit = offset % 8;

        // Touch only the single header byte that owns this slot.
        let mut byte: u8 = read_generic(core, base_address + index)?;

        debug_assert!(byte & (1 << bit) == 0);

        byte |= 1 << bit;
        write_generic(core, base_address + index, byte)?;

        Ok(())
    }

    #[inline]
    fn find_bucket_index(size: u32) -> usize {
        let size = size.max(BUCKETS[0] as u32);
        (size.ilog2() + if size.is_power_of_two() { 0 } else { 1 } - 2) as _
    }
}

#[cfg(test)]
mod tests {
    use wie_util::{ByteRead, Result};

    use crate::ArmCore;

    use super::BucketAllocator;

    // Bucket 0 (4-byte) header_length = BUCKET_SIZE / 4 / 8 = 0x80000.
    // First 4-byte slot starts at base + 0x80000 = 0x40080000.
    // Bucket 1 (8-byte) lives at base + BUCKET_SIZE = 0x41000000;
    // header_length = BUCKET_SIZE / 8 / 8 = 0x40000, so first slot = 0x41040000.

    #[test]
    fn test_allocator() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x8000000)?;

        BucketAllocator::init(&mut core, 0x40000000, 0x8000000)?;

        let address1 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address1, 0x40080000);

        let address2 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address2, 0x40080004);

        let address3 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address3, 0x40080008);

        BucketAllocator::free(&mut core, 0x40000000, address2, 4)?;

        let address4 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address4, 0x40080004);

        let address5 = BucketAllocator::alloc(&mut core, 0x40000000, 5)?;
        assert_eq!(address5, 0x41040000);

        let address6 = BucketAllocator::alloc(&mut core, 0x40000000, 6)?;
        assert_eq!(address6, 0x41040008);

        let address7 = BucketAllocator::alloc(&mut core, 0x40000000, 7)?;
        assert_eq!(address7, 0x41040010);

        let address8 = BucketAllocator::alloc(&mut core, 0x40000000, 8)?;
        assert_eq!(address8, 0x41040018);

        Ok(())
    }

    // Regression: each bucket's slot region used to extend past `BUCKET_SIZE`
    // into the next bucket's region (header_length bytes of overlap), so two
    // different size classes could hand out the same address. After init, the
    // header bits for slots that would land outside the bucket's own region
    // must be 0 (already-allocated) so they're never returned by alloc().
    #[test]
    fn test_init_clamps_slots_to_region() -> Result<()> {
        const BUCKET_SIZE: u32 = 0x1000000;
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x8000000)?;
        BucketAllocator::init(&mut core, 0x40000000, 0x8000000)?;

        // For each bucket, walk the header from the end backwards: the trailing
        // bits that correspond to addresses past `bucket_base + BUCKET_SIZE`
        // must all be 0. Concretely, `usable_slots` = (BUCKET_SIZE - header)
        // / bucket; bits at index >= usable_slots must be cleared.
        let buckets: [u32; 8] = [4, 8, 16, 32, 64, 128, 256, 512];
        for (i, bucket) in buckets.into_iter().enumerate() {
            let bucket_base = 0x40000000 + i as u32 * BUCKET_SIZE;
            let header_length = BUCKET_SIZE / bucket / 8;
            let usable_slots = (BUCKET_SIZE - header_length) / bucket;

            // First slot that should be marked as already-allocated.
            for slot in usable_slots..header_length * 8 {
                let byte_idx = slot / 8;
                let bit = slot % 8;
                let mut byte = [0u8; 1];
                core.read_bytes(bucket_base + byte_idx, &mut byte).unwrap();
                assert!(
                    byte[0] & (1 << bit) == 0,
                    "bucket {bucket} slot {slot} (byte {byte_idx} bit {bit}) is free but lands outside region"
                );

                // Sanity: also confirm that this slot's address would indeed
                // overlap the next bucket's region.
                let slot_addr = bucket_base + header_length + slot * bucket;
                let next_bucket_base = 0x40000000 + (i as u32 + 1) * BUCKET_SIZE;
                assert!(slot_addr >= next_bucket_base || i == buckets.len() - 1);
            }
        }
        Ok(())
    }

    #[test]
    fn test_init_rejects_undersized_region() {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000000).unwrap();

        // 0x1000000 is one BUCKET_SIZE — far too small for 8 buckets.
        assert!(BucketAllocator::init(&mut core, 0x40000000, 0x1000000).is_err());
    }

    #[test]
    fn test_allocator_small_sizes() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x8000000)?;

        BucketAllocator::init(&mut core, 0x40000000, 0x8000000)?;

        // sizes 0..=3 must all land in the smallest (4-byte) bucket
        let a0 = BucketAllocator::alloc(&mut core, 0x40000000, 0)?;
        assert_eq!(a0, 0x40080000);
        let a1 = BucketAllocator::alloc(&mut core, 0x40000000, 1)?;
        assert_eq!(a1, 0x40080004);
        let a2 = BucketAllocator::alloc(&mut core, 0x40000000, 2)?;
        assert_eq!(a2, 0x40080008);
        let a3 = BucketAllocator::alloc(&mut core, 0x40000000, 3)?;
        assert_eq!(a3, 0x4008000c);

        // free with the same (small) size must round-trip without panicking
        BucketAllocator::free(&mut core, 0x40000000, a1, 1)?;
        let a1b = BucketAllocator::alloc(&mut core, 0x40000000, 1)?;
        assert_eq!(a1b, 0x40080004);

        Ok(())
    }
}
