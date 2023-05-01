use std::collections::BTreeMap;

use crate::util;

use super::{ArmCore, HEAP_BASE};

const ALIGN: u32 = 4;

pub struct Allocator {
    base: u32,
    map: BTreeMap<u32, u32>,
}

impl Allocator {
    pub fn new(core: &mut ArmCore) -> anyhow::Result<Self> {
        let size = 0x10000;

        core.alloc(HEAP_BASE, size)?;

        let map = BTreeMap::from_iter(vec![(size, 0)]);

        Ok(Self { base: HEAP_BASE, map })
    }

    pub fn alloc(&mut self, size: u32) -> Option<u32> {
        let size = if size == 0 { ALIGN } else { size };
        let address = self.find_address(size)?;

        self.map.insert(address, size);

        Some(self.base + address)
    }

    #[allow(dead_code)]
    pub fn free(&mut self, address: u32) {
        self.map.remove(&address).unwrap();
    }

    fn find_address(&self, request_size: u32) -> Option<u32> {
        let mut cursor = 0;
        for (address, size) in self.map.iter() {
            if address - cursor >= request_size {
                return Some(cursor);
            } else {
                cursor = util::round_up((address + size) as usize, ALIGN as usize) as u32;
            }
        }

        None
    }
}
