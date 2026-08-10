mod bucket;
mod list;

use wie_util::Result;

use crate::{
    ArmCore,
    core::{HEAP_BASE, HEAP_SIZE},
};

use self::{
    bucket::{BUCKET_MAX, BucketAllocator},
    list::ListAllocator,
};

pub struct Allocator;

impl Allocator {
    pub fn init(core: &mut ArmCore) -> Result<()> {
        core.map(HEAP_BASE, HEAP_SIZE)?;

        ListAllocator::init(core, HEAP_BASE, HEAP_SIZE / 2)?;
        BucketAllocator::init(core, HEAP_BASE + HEAP_SIZE / 2, HEAP_SIZE / 2)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, size: u32) -> Result<u32> {
        if size > BUCKET_MAX as _ {
            ListAllocator::alloc(core, HEAP_BASE, HEAP_SIZE / 2, size)
        } else {
            BucketAllocator::alloc(core, HEAP_BASE + HEAP_SIZE / 2, size)
        }
    }

    pub fn free(core: &mut ArmCore, address: u32, size: u32) -> Result<()> {
        if size > BUCKET_MAX as _ {
            ListAllocator::free(core, address)
        } else {
            BucketAllocator::free(core, HEAP_BASE + HEAP_SIZE / 2, address, size)
        }
    }

    pub fn is_allocated(core: &ArmCore, address: u32, size: u32) -> Result<bool> {
        if size > BUCKET_MAX as _ {
            ListAllocator::is_allocated(core, HEAP_BASE, HEAP_SIZE / 2, address, size)
        } else {
            BucketAllocator::is_allocated(core, HEAP_BASE + HEAP_SIZE / 2, address, size)
        }
    }
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use crate::{Allocator, ArmCore};

    #[test]
    fn allocation_status_tracks_bucket_and_list_allocations() -> Result<()> {
        let mut core = ArmCore::new(false, None)?;
        Allocator::init(&mut core)?;

        let bucket = Allocator::alloc(&mut core, 12)?;
        let list = Allocator::alloc(&mut core, 1024)?;
        assert!(Allocator::is_allocated(&core, bucket, 12)?);
        assert!(Allocator::is_allocated(&core, list, 1024)?);
        assert!(!Allocator::is_allocated(&core, bucket + 4, 12)?);

        Allocator::free(&mut core, bucket, 12)?;
        Allocator::free(&mut core, list, 1024)?;
        assert!(!Allocator::is_allocated(&core, bucket, 12)?);
        assert!(!Allocator::is_allocated(&core, list, 1024)?);

        Ok(())
    }
}
