use alloc::{collections::BTreeMap, vec, vec::Vec};

use armv4t_emu::{reg, Cpu, Memory, Mode};

use crate::engine::{ArmEngine, ArmEngineResult, ArmRegister, MemoryPermission};

pub struct Armv4tEmuEngine {
    cpu: Cpu,
    mem: Armv4tEmuMemory,
}

impl Armv4tEmuEngine {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: Armv4tEmuMemory::new(),
        }
    }
}

impl ArmEngine for Armv4tEmuEngine {
    fn run(&mut self, end: u32, hook: core::ops::Range<u32>) -> ArmEngineResult<()> {
        loop {
            let pc = self.cpu.reg_get(Mode::User, reg::PC);
            if pc == end || hook.contains(&pc) {
                break;
            }

            self.cpu.step(&mut self.mem);
        }

        Ok(())
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

    fn mem_write(&mut self, address: u32, data: &[u8]) -> ArmEngineResult<()> {
        self.mem.write_range(address, data);

        Ok(())
    }

    fn mem_read(&mut self, address: u32, size: usize) -> ArmEngineResult<Vec<u8>> {
        let result = self.mem.read_range(address, size);

        Ok(result)
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

// TODO we can optimize it..
struct Armv4tEmuMemory {
    data: BTreeMap<u32, Vec<u8>>,
}

impl Armv4tEmuMemory {
    #[allow(dead_code)]
    fn new() -> Self {
        Self { data: BTreeMap::new() }
    }

    fn map(&mut self, address: u32, size: usize) {
        let page_start = address & !0xffff;
        let page_end = (address + size as u32 + 0xffff) & !0xffff;

        for page in (page_start..page_end).step_by(0x10000) {
            self.data.entry(page).or_insert_with(|| vec![0; 0x10000]);
        }
    }

    fn read_range(&self, address: u32, size: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(size);
        let mut remaining_size = size;
        let mut current_address = address;

        while remaining_size > 0 {
            let page_address = current_address & !0xffff;
            let page_data = self.data.get(&page_address).unwrap();
            let offset = (current_address - page_address) as usize;
            let available_bytes = (0x10000 - offset).min(remaining_size);

            result.extend_from_slice(&page_data[offset..offset + available_bytes]);
            remaining_size -= available_bytes;
            current_address += available_bytes as u32;
        }

        result
    }

    fn write_range(&mut self, address: u32, data: &[u8]) {
        let mut current_address = address;
        let mut data_index = 0;

        while data_index < data.len() {
            let page_address = current_address & !0xffff;
            let page_data = self.data.get_mut(&page_address).unwrap();
            let offset = (current_address - page_address) as usize;
            let available_bytes = (0x10000 - offset).min(data.len() - data_index);

            page_data[offset..offset + available_bytes].copy_from_slice(&data[data_index..data_index + available_bytes]);
            data_index += available_bytes;
            current_address += available_bytes as u32;
        }
    }
}

impl Memory for Armv4tEmuMemory {
    fn r8(&mut self, addr: u32) -> u8 {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        self.data.get(&page).unwrap()[offset as usize]
    }

    fn r16(&mut self, addr: u32) -> u16 {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        let data = self.data.get(&page).unwrap();

        (data[offset as usize] as u16) | ((data[offset as usize + 1] as u16) << 8)
    }

    fn r32(&mut self, addr: u32) -> u32 {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        let data = self.data.get(&page).unwrap();

        (data[offset as usize] as u32)
            | ((data[offset as usize + 1] as u32) << 8)
            | ((data[offset as usize + 2] as u32) << 16)
            | ((data[offset as usize + 3] as u32) << 24)
    }

    fn w8(&mut self, addr: u32, val: u8) {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        self.data.get_mut(&page).unwrap()[offset as usize] = val;
    }

    fn w16(&mut self, addr: u32, val: u16) {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        let data = self.data.get_mut(&page).unwrap();

        data[offset as usize] = val as u8;
        data[offset as usize + 1] = (val >> 8) as u8;
    }

    fn w32(&mut self, addr: u32, val: u32) {
        let page = addr & !0xffff;
        let offset = addr & 0xffff;

        let data = self.data.get_mut(&page).unwrap();

        data[offset as usize] = val as u8;
        data[offset as usize + 1] = (val >> 8) as u8;
        data[offset as usize + 2] = (val >> 16) as u8;
        data[offset as usize + 3] = (val >> 24) as u8;
    }
}
