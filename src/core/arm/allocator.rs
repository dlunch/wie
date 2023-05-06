use std::mem::size_of;

use crate::util::round_up;

use super::{ArmCore, HEAP_BASE};

const HEAP_SIZE: u32 = 0x100000;

#[derive(Clone, Copy)]
struct AllocationHeader {
    size: u32,
    in_use: u32,
}

pub struct Allocator {}

impl Allocator {
    pub fn init(core: &mut ArmCore) -> anyhow::Result<()> {
        core.alloc(HEAP_BASE, HEAP_SIZE)?;

        let header = AllocationHeader { size: HEAP_SIZE, in_use: 0 };

        core.write(HEAP_BASE, header)?;

        Ok(())
    }

    pub fn alloc(core: &mut ArmCore, size: u32) -> anyhow::Result<u32> {
        let alloc_size = round_up(size as usize + size_of::<AllocationHeader>(), 4) as u32;

        let address = Self::find_address(core, alloc_size).ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;

        let previous_header = core.read::<AllocationHeader>(address)?;

        let header = AllocationHeader { size: alloc_size, in_use: 1 };
        core.write(address, header)?;

        // write next
        if previous_header.size > alloc_size {
            let next_header = AllocationHeader {
                size: previous_header.size - alloc_size,
                in_use: 0,
            };
            core.write(address + alloc_size, next_header)?;
        }

        Ok(address + 8)
    }

    pub fn free(core: &mut ArmCore, address: u32) -> anyhow::Result<()> {
        let base_address = address - 8;

        let header = core.read::<AllocationHeader>(base_address)?;
        assert!(header.in_use == 1);

        let header = AllocationHeader {
            size: header.size,
            in_use: 0,
        };
        core.write(base_address, header)?;

        Ok(())
    }

    fn find_address(core: &mut ArmCore, request_size: u32) -> Option<u32> {
        let mut cursor = HEAP_BASE;
        loop {
            let header = core.read::<AllocationHeader>(cursor).ok()?;
            if header.in_use == 0 && header.size >= request_size {
                return Some(cursor);
            } else {
                cursor += header.size;
            }

            if cursor >= HEAP_BASE + HEAP_SIZE {
                break;
            }
        }

        None
    }
}
