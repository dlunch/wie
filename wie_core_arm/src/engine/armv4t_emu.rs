use alloc::{collections::BTreeMap, vec, vec::Vec};

use armv4t_emu::{reg, Cpu, Memory, Mode};

use crate::engine::{ArmEngine, ArmEngineResult, ArmRegister, MemoryPermission};

pub struct Armv4tEmuEngine {
    cpu: Cpu,
    mem: Armv4tEmuMemory,
}

impl Armv4tEmuEngine {
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
        // TODO change to block write
        for (i, byte) in data.iter().enumerate() {
            self.mem.w8(address + i as u32, *byte);
        }

        Ok(())
    }

    fn mem_read(&mut self, address: u32, size: usize) -> ArmEngineResult<alloc::vec::Vec<u8>> {
        // TODO change to block read

        let mut result = Vec::new();
        for i in 0..size {
            result.push(self.mem.r8(address + i as u32));
        }

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
