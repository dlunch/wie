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
}
