use alloc::format;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_util::{read_generic, write_generic, Result, WieError};

use crate::core::ArmCore;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct ListAllocationHeader {
    data: u32,
}

impl ListAllocationHeader {
    pub fn new(size: u32, in_use: bool) -> Self {
        Self {
            data: size | (in_use as u32) << 31,
        }
    }

    pub fn size(&self) -> u32 {
        self.data & 0x7FFFFFFF
    }

    pub fn in_use(&self) -> bool {
        self.data & 0x80000000 != 0
    }
}

pub struct ListAllocator;

impl ListAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, size: u32) -> Result<()> {
        let header = ListAllocationHeader::new(size, false);

        write_generic(core, base_address, header)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let size_to_alloc = (size as usize + size_of::<ListAllocationHeader>()).next_multiple_of(4) as u32;

        let address = Self::find_address(core, base_address, base_size, size_to_alloc)?;

        let previous_header: ListAllocationHeader = read_generic(core, address)?;

        let header = ListAllocationHeader::new(size_to_alloc, true);
        write_generic(core, address, header)?;

        // write next
        if previous_header.size() > size_to_alloc {
            let next_header = ListAllocationHeader::new(previous_header.size() - size_to_alloc, false);
            write_generic(core, address + size_to_alloc, next_header)?;
        }

        tracing::trace!("Allocated {:#x} bytes at {:#x}", size, address + size_of::<ListAllocationHeader>() as u32);

        Ok(address + size_of::<ListAllocationHeader>() as u32)
    }

    pub fn free(core: &mut ArmCore, address: u32) -> Result<()> {
        let base_address = address - size_of::<ListAllocationHeader>() as u32;

        tracing::trace!("Freeing {:#x}", address);

        let header: ListAllocationHeader = read_generic(core, base_address)?;
        assert!(header.in_use());

        let header = ListAllocationHeader::new(header.size(), false);
        write_generic(core, base_address, header)?;

        Ok(())
    }

    fn find_address(core: &ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let mut cursor = base_address;
        loop {
            let header: ListAllocationHeader = read_generic(core, cursor)?;
            if header.size() == 0 {
                return Err(WieError::FatalError(format!("Invalid allocation header at {:#x}", cursor)));
            }

            if !header.in_use() && header.size() >= size {
                return Ok(cursor);
            } else {
                cursor += header.size();
            }

            if cursor >= base_address + base_size {
                break;
            }
        }

        Err(WieError::AllocationFailure)
    }
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use crate::ArmCore;

    use super::ListAllocator;

    #[test]
    fn test_allocator() -> Result<()> {
        let mut core = ArmCore::new().unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        assert_eq!(address, 0x40000004);

        Ok(())
    }
}
