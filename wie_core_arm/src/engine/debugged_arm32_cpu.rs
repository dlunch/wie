use core::ops::Range;

use super::{Arm32CpuEngine, ArmEngine};

pub struct DebuggedArm32CpuEngine {
    arm32_cpu_engine: Arm32CpuEngine,
}

impl DebuggedArm32CpuEngine {
    pub fn new() -> Self {
        Self {
            arm32_cpu_engine: Arm32CpuEngine::new(),
        }
    }
}

impl ArmEngine for DebuggedArm32CpuEngine {
    fn run(&mut self, end: u32, hook: Range<u32>, count: u32) -> wie_util::Result<u32> {
        self.arm32_cpu_engine.run(end, hook, count)
    }

    fn reg_write(&mut self, reg: super::ArmRegister, value: u32) {
        self.arm32_cpu_engine.reg_write(reg, value)
    }

    fn reg_read(&self, reg: super::ArmRegister) -> u32 {
        self.arm32_cpu_engine.reg_read(reg)
    }

    fn mem_map(&mut self, address: u32, size: usize, permission: super::MemoryPermission) {
        self.arm32_cpu_engine.mem_map(address, size, permission)
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> wie_util::Result<()> {
        self.arm32_cpu_engine.mem_write(address, data)
    }

    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> wie_util::Result<usize> {
        self.arm32_cpu_engine.mem_read(address, size, result)
    }

    fn is_mapped(&self, address: u32, size: usize) -> bool {
        self.arm32_cpu_engine.is_mapped(address, size)
    }
}
