use alloc::format;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_util::{Result, WieError, read_generic, write_generic};

use crate::core::ArmCore;

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct ListAllocationHeader {
    data: u32,
}

impl ListAllocationHeader {
    pub fn new(size: u32, in_use: bool) -> Self {
        Self {
            data: size | ((in_use as u32) << 31),
        }
    }

    pub fn size(&self) -> u32 {
        self.data & 0x7FFFFFFF
    }

    pub fn in_use(&self) -> bool {
        self.data & 0x80000000 != 0
    }
}

const CANARY_SIZE: u32 = 4;
const CANARY_VALUE: u32 = 0xDEADBEEF;

pub struct ListAllocator;

impl ListAllocator {
    pub fn init(core: &mut ArmCore, base_address: u32, size: u32) -> Result<()> {
        let header = ListAllocationHeader::new(size, false);

        write_generic(core, base_address, header)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let size_to_alloc = (size as usize + size_of::<ListAllocationHeader>()).next_multiple_of(4) as u32 + CANARY_SIZE;

        let address = Self::find_address(core, base_address, base_size, size_to_alloc)?;

        let previous_header: ListAllocationHeader = read_generic(core, address)?;

        let header = ListAllocationHeader::new(size_to_alloc, true);
        write_generic(core, address, header)?;

        // write next
        if previous_header.size() > size_to_alloc {
            let next_header = ListAllocationHeader::new(previous_header.size() - size_to_alloc, false);
            write_generic(core, address + size_to_alloc, next_header)?;
        }

        // write canary
        write_generic(core, address + size_to_alloc - CANARY_SIZE, CANARY_VALUE)?;

        tracing::trace!("Allocated {size:#x} bytes at {:#x}", address + size_of::<ListAllocationHeader>() as u32);

        Ok(address + size_of::<ListAllocationHeader>() as u32)
    }

    pub fn free(core: &mut ArmCore, address: u32) -> Result<()> {
        let base_address = address - size_of::<ListAllocationHeader>() as u32;

        tracing::trace!("Freeing {address:#x}");

        let header: ListAllocationHeader = read_generic(core, base_address)?;
        if !header.in_use() {
            return Err(WieError::FatalError(format!("Double free at {address:#x}")));
        }

        let canary_value: u32 = read_generic(core, base_address + header.size() - CANARY_SIZE)?;
        if canary_value != CANARY_VALUE {
            return Err(WieError::FatalError(format!(
                "Invalid canary value at {base_address:#x}: expected {CANARY_VALUE:#x}, got {canary_value:#x}"
            )));
        }

        let header = ListAllocationHeader::new(header.size(), false);
        write_generic(core, base_address, header)?;

        Ok(())
    }

    fn find_address(core: &mut ArmCore, base_address: u32, base_size: u32, size: u32) -> Result<u32> {
        let end = base_address + base_size;
        let mut cursor = base_address;
        loop {
            let mut header: ListAllocationHeader = read_generic(core, cursor)?;
            if header.size() == 0 {
                return Err(WieError::FatalError(format!("Invalid allocation header at {cursor:#x}")));
            }

            if !header.in_use() {
                loop {
                    let next = cursor + header.size();
                    if next >= end {
                        break;
                    }
                    let next_header: ListAllocationHeader = read_generic(core, next)?;
                    if next_header.in_use() || next_header.size() == 0 {
                        break;
                    }
                    header = ListAllocationHeader::new(header.size() + next_header.size(), false);
                    write_generic(core, cursor, header)?;
                }
                if header.size() >= size {
                    return Ok(cursor);
                }
            }

            cursor += header.size();
            if cursor >= end {
                break;
            }
        }

        Err(WieError::AllocationFailure)
    }
}

#[cfg(test)]
mod tests {
    use wie_util::{Result, WieError, write_generic};

    use crate::ArmCore;

    use super::ListAllocator;

    #[test]
    fn test_allocator() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        assert_eq!(address, 0x40000004);

        Ok(())
    }

    #[test]
    fn test_coalesce_adjacent_free_blocks() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x400)?;
        let a = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x100)?;
        let b = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x100)?;
        let _c = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x1e8)?;

        ListAllocator::free(&mut core, a)?;
        ListAllocator::free(&mut core, b)?;

        let merged = ListAllocator::alloc(&mut core, 0x40000000, 0x400, 0x150)?;
        assert_eq!(merged, a);

        Ok(())
    }

    #[test]
    fn test_double_free_returns_error() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        ListAllocator::free(&mut core, address)?;
        let result = ListAllocator::free(&mut core, address);

        assert!(matches!(result, Err(WieError::FatalError(_))));

        Ok(())
    }

    #[test]
    fn test_corrupted_canary_returns_error() -> Result<()> {
        let mut core = ArmCore::new(false, None).unwrap();
        core.map(0x40000000, 0x1000)?;

        ListAllocator::init(&mut core, 0x40000000, 0x1000)?;
        let address = ListAllocator::alloc(&mut core, 0x40000000, 0x1000, 4)?;

        write_generic(&mut core, address + 4, 0u32)?;
        let result = ListAllocator::free(&mut core, address);

        assert!(matches!(result, Err(WieError::FatalError(_))));

        Ok(())
    }
}
