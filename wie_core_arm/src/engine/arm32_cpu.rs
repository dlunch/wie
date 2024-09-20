use alloc::{boxed::Box, sync::Arc};
use core::{array, ops::Range};

use arm32_cpu::{reg, Cpu, Memory, Mode};
use spin::Mutex;

use wie_util::{Result, WieError};

use crate::engine::{ArmEngine, ArmRegister, MemoryPermission};

pub struct Arm32CpuEngine {
    cpu: Cpu,
    mem: Armv4tEmuMemory,
    memory_error: Arc<Mutex<Option<u32>>>,
}

impl Arm32CpuEngine {
    pub fn new() -> Self {
        let memory_error = Arc::new(Mutex::new(None));
        Self {
            cpu: Cpu::new(),
            mem: Armv4tEmuMemory::new(memory_error.clone()),
            memory_error,
        }
    }
}

impl ArmEngine for Arm32CpuEngine {
    fn run(&mut self, end: u32, hook: Range<u32>, mut count: u32) -> Result<u32> {
        loop {
            let pc = self.cpu.reg_get(Mode::User, reg::PC);
            if pc < 0x1000 {
                return Err(WieError::InvalidMemoryAccess(pc));
            }

            if pc == end || hook.contains(&pc) || count == 0 {
                return Ok(pc);
            }

            self.cpu.step(&mut self.mem);
            count -= 1;

            if let Some(x) = *self.memory_error.lock() {
                return Err(WieError::InvalidMemoryAccess(x));
            }
        }
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
        if reg == ArmRegister::PC && value % 2 == 1 {
            self.cpu.reg_set(Mode::User, reg.into_armv4t(), value - 1);

            let cpsr = self.cpu.reg_get(Mode::User, reg::CPSR);
            self.cpu.reg_set(Mode::User, reg::CPSR, cpsr | (1 << 5)); // T bit

            return;
        }
        self.cpu.reg_set(Mode::User, reg.into_armv4t(), value);
    }

    fn reg_read(&self, reg: ArmRegister) -> u32 {
        self.cpu.reg_get(Mode::User, reg.into_armv4t())
    }

    fn mem_map(&mut self, address: u32, size: usize, _permission: MemoryPermission) {
        self.mem.map(address, size);
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()> {
        self.mem.write_range(address, data);

        Ok(())
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize> {
        Ok(self.mem.read_range(address, size, result))
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.mem.is_mapped(address, size)
    }
}

impl ArmRegister {
    fn into_armv4t(self) -> u8 {
        match self {
            ArmRegister::R0 => 0,
            ArmRegister::R1 => 1,
            ArmRegister::R2 => 2,
            ArmRegister::R3 => 3,
            ArmRegister::R4 => 4,
            ArmRegister::R5 => 5,
            ArmRegister::R6 => 6,
            ArmRegister::R7 => 7,
            ArmRegister::R8 => 8,
            ArmRegister::SB => 9,
            ArmRegister::SL => 10,
            ArmRegister::FP => 11,
            ArmRegister::IP => 12,
            ArmRegister::SP => reg::SP,
            ArmRegister::LR => reg::LR,
            ArmRegister::PC => reg::PC,
            ArmRegister::Cpsr => reg::CPSR,
        }
    }
}

const TOTAL_MEMORY: usize = 0xffffffff;
const PAGE_SIZE: usize = 0x10000;
const PAGE_MASK: u32 = (PAGE_SIZE - 1) as _;

struct Armv4tEmuMemory {
    pages: [Option<Box<Mutex<[u8; PAGE_SIZE]>>>; TOTAL_MEMORY / PAGE_SIZE],
    memory_error: Arc<Mutex<Option<u32>>>,
}

impl Armv4tEmuMemory {
    fn new(memory_error: Arc<Mutex<Option<u32>>>) -> Self {
        Self {
            pages: array::from_fn(|_| None),
            memory_error,
        }
    }

    fn map(&mut self, address: u32, size: usize) {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            let page_data = &mut self.pages[page as usize / PAGE_SIZE];
            if page_data.is_none() {
                *page_data = Some(Box::new(Mutex::new([0; PAGE_SIZE])));
            }
        }
    }

    fn read_range(&self, address: u32, size: usize, result: &mut [u8]) -> usize {
        let mut remaining_size = size;
        let mut current_address = address;

        while remaining_size > 0 {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE].as_ref().unwrap();
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(remaining_size);

            result[size - remaining_size..size - remaining_size + available_bytes]
                .copy_from_slice(&page_data.lock()[offset..offset + available_bytes]);
            remaining_size -= available_bytes;
            current_address += available_bytes as u32;
        }

        size
    }

    fn write_range(&mut self, address: u32, data: &[u8]) {
        let mut current_address = address;
        let mut data_index = 0;

        while data_index < data.len() {
            let page_address = current_address & !PAGE_MASK;
            let page_data = self.pages[page_address as usize / PAGE_SIZE].as_mut().unwrap();
            let offset = (current_address - page_address) as usize;
            let available_bytes = (PAGE_SIZE - offset).min(data.len() - data_index);

            page_data.lock()[offset..offset + available_bytes].copy_from_slice(&data[data_index..data_index + available_bytes]);
            data_index += available_bytes;
            current_address += available_bytes as u32;
        }
    }

    fn get_page(&mut self, addr: u32) -> Option<&Mutex<[u8; PAGE_SIZE]>> {
        let page_address = addr & !PAGE_MASK;
        let page_data = self.pages[page_address as usize / PAGE_SIZE].as_mut();

        if let Some(x) = page_data {
            Some(x)
        } else {
            *self.memory_error.lock() = Some(addr);
            None
        }
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        let page_start = address & !PAGE_MASK;
        let page_end = (address + size as u32 + PAGE_MASK) & !PAGE_MASK;

        if self.pages[page_start as usize / PAGE_SIZE].is_none() {
            return false;
        }

        for page in (page_start..page_end).step_by(PAGE_SIZE) {
            if self.pages[page as usize / PAGE_SIZE].is_none() {
                return false;
            }
        }

        true
    }
}

impl Memory for Armv4tEmuMemory {
    fn r8(&mut self, addr: u32) -> u8 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap().lock();

        data[offset as usize]
    }

    fn r16(&mut self, addr: u32) -> u16 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap().lock();

        (data[offset as usize] as u16) | ((data[offset as usize + 1] as u16) << 8)
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return 0;
        }

        let data = page.unwrap().lock();
        (data[offset as usize] as u32)
            | ((data[offset as usize + 1] as u32) << 8)
            | ((data[offset as usize + 2] as u32) << 16)
            | ((data[offset as usize + 3] as u32) << 24)
    }

    fn w8(&mut self, addr: u32, val: u8) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let mut data = page.unwrap().lock();

        data[offset as usize] = val;
    }

    fn w16(&mut self, addr: u32, val: u16) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let mut data = page.unwrap().lock();

        data[offset as usize] = val as u8;
        data[offset as usize + 1] = (val >> 8) as u8;
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let offset = addr & PAGE_MASK;

        let page = self.get_page(addr);
        if page.is_none() {
            return;
        }

        let mut data = page.unwrap().lock();

        data[offset as usize] = val as u8;
        data[offset as usize + 1] = (val >> 8) as u8;
        data[offset as usize + 2] = (val >> 16) as u8;
        data[offset as usize + 3] = (val >> 24) as u8;
    }
}

#[cfg(test)]
mod tests {
    use alloc::sync::Arc;

    use arm32_cpu::Memory;

    use super::Armv4tEmuMemory;

    #[test]
    fn test_memory_basic() {
        let mut memory = Armv4tEmuMemory::new(Arc::new(Default::default()));

        memory.map(0x10000, 0x1000);
        memory.map(0x11000, 0x1000);
        memory.map(0x20000, 0x10000);

        memory.write_range(0x10000, &[123; 0x1000]);

        let mut buf = [0; 0x1000];
        memory.read_range(0x10000, 0x1000, &mut buf);
        assert_eq!(buf, [123; 0x1000]);

        memory.write_range(0x10900, &[100; 0x1000]);

        memory.read_range(0x10900, 0x1000, &mut buf);
        assert_eq!(buf, [100; 0x1000]);

        let r8 = memory.r8(0x10000);
        assert_eq!(r8, 123);

        let r16 = memory.r16(0x10000);
        assert_eq!(r16, 123 | (123 << 8));

        let r32 = memory.r32(0x10000);
        assert_eq!(r32, 123 | (123 << 8) | (123 << 16) | (123 << 24));

        memory.w8(0x10000, 12);
        let r8 = memory.r8(0x10000);
        assert_eq!(r8, 12);

        memory.w16(0x10000, 0x1234);
        let r16 = memory.r16(0x10000);
        assert_eq!(r16, 0x1234);

        memory.w32(0x10000, 0x12345678);
        let r32 = memory.r32(0x10000);
        assert_eq!(r32, 0x12345678);
    }

    #[test]
    #[should_panic]
    fn test_memory_unmapped_read() {
        let mut memory = Armv4tEmuMemory::new(Arc::new(Default::default()));

        memory.map(0x10000, 0x10000);

        let mut buf = [0; 0x1000];
        memory.read_range(0x1f500, 0x1000, &mut buf);
    }

    #[test]
    #[should_panic]
    fn test_memory_unmapped_write() {
        let mut memory = Armv4tEmuMemory::new(Arc::new(Default::default()));

        memory.map(0x10000, 0x10000);

        memory.write_range(0x1f500, &[12; 0x1000]);
    }
}