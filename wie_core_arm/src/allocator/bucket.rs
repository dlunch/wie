use alloc::vec;

use wie_util::{ByteRead, ByteWrite};

use crate::{core::ArmCore, ArmCoreError, ArmCoreResult};

pub const BUCKET_MAX: usize = 512;
const BUCKETS: [usize; 8] = [4, 8, 16, 32, 64, 128, 256, 512];
const BUCKET_SIZE: usize = 0x100000;

pub struct BucketAllocator;

impl BucketAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, _base_size: u32) -> ArmCoreResult<()> {
        // initialize each bucket header with ones
        // header contains bitset of allocation, 1 is unallocated, 0 is allocated

        for (i, bucket) in BUCKETS.into_iter().enumerate() {
            let header_length = BUCKET_SIZE / bucket / 8;
            let header_address = base_address + i as u32 * BUCKET_SIZE as u32;

            tracing::info!("Bucket {} header size {}", bucket, BUCKET_SIZE / bucket / 8);

            core.write_bytes(header_address, &vec![0xff; header_length])?;
        }

        Ok(())
    }
    pub fn alloc(core: &mut ArmCore, base_address: u32, size: u32) -> ArmCoreResult<u32> {
        let bucket_index = Self::find_bucket_index(size);
        let bucket = BUCKETS[bucket_index];
        let base_address = base_address + (bucket_index * BUCKET_SIZE) as u32;
        let header_length = BUCKET_SIZE / bucket / 8;

        let mut header = vec![0u8; header_length];
        core.read_bytes(base_address, header_length as _, &mut header)?;

        for (i, item) in header.iter_mut().enumerate() {
            if *item == 0 {
                continue;
            }

            let bit = item.trailing_zeros();
            *item &= !(1 << bit);
            let address = base_address + (header_length as u32) + (i as u32 * 8 + bit) * bucket as u32;

            core.write_bytes(base_address + i as u32, &[*item])?;

            return Ok(address);
        }

        Err(ArmCoreError::AllocationFailure)
    }

    pub fn free(core: &mut ArmCore, base_address: u32, address: u32, size: u32) -> ArmCoreResult<()> {
        let bucket_index = Self::find_bucket_index(size);
        let bucket = BUCKETS[bucket_index];
        let base_address = base_address + (bucket_index * BUCKET_SIZE) as u32;
        let header_length = BUCKET_SIZE / bucket / 8;

        let mut header = vec![0u8; header_length];
        core.read_bytes(base_address, header_length as _, &mut header)?;

        let offset = (address - base_address - header_length as u32) / bucket as u32;
        let index = offset / 8;
        let bit = offset % 8;

        debug_assert!(header[index as usize] & (1 << bit) == 0);

        header[index as usize] |= 1 << bit;

        core.write_bytes(base_address + index, &[header[index as usize]])?;

        Ok(())
    }

    #[inline]
    fn find_bucket_index(size: u32) -> usize {
        (size.ilog2() + if size.is_power_of_two() { 0 } else { 1 } - 2) as _
    }
}

#[cfg(test)]
mod tests {
    use crate::{ArmCore, ArmCoreResult};

    use super::BucketAllocator;

    #[test]
    fn test_allocator() -> ArmCoreResult<()> {
        let mut core = ArmCore::new().unwrap();
        core.map(0x40000000, 0x1000000)?;

        BucketAllocator::init(&mut core, 0x40000000, 0x1000000)?;

        let address1 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address1, 0x40008000);

        let address2 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address2, 0x40008004);

        let address3 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address3, 0x40008008);

        BucketAllocator::free(&mut core, 0x40000000, address2, 4)?;

        let address4 = BucketAllocator::alloc(&mut core, 0x40000000, 4)?;
        assert_eq!(address4, 0x40008004);

        let address5 = BucketAllocator::alloc(&mut core, 0x40000000, 5)?;
        assert_eq!(address5, 0x40104000);

        let address6 = BucketAllocator::alloc(&mut core, 0x40000000, 6)?;
        assert_eq!(address6, 0x40104008);

        let address7 = BucketAllocator::alloc(&mut core, 0x40000000, 7)?;
        assert_eq!(address7, 0x40104010);

        let address8 = BucketAllocator::alloc(&mut core, 0x40000000, 8)?;
        assert_eq!(address8, 0x40104018);

        Ok(())
    }
}
