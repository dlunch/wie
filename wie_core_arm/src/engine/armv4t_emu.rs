use alloc::vec::Vec;

use armv4t_emu::{reg, Cpu, ExampleMem, Memory, Mode};

use crate::engine::{ArmEngine, ArmEngineResult, ArmRegister, MemoryPermission};

pub struct Armv4tEmuEngine {
    cpu: Cpu,
    mem: ExampleMem,
}

impl Armv4tEmuEngine {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            mem: ExampleMem::new(),
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

    fn mem_map(&mut self, _address: u32, _size: usize, _permission: MemoryPermission) {
        // TODO
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
