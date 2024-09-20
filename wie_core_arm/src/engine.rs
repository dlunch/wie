mod arm32_cpu;

use core::ops::Range;

use wie_util::Result;

pub use arm32_cpu::Arm32CpuEngine;

pub trait ArmEngine: Sync + Send {
    fn run(&mut self, end: u32, hook: Range<u32>, count: u32) -> Result<u32>;
    fn reg_write(&mut self, reg: ArmRegister, value: u32);
    fn reg_read(&self, reg: ArmRegister) -> u32;
    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission);
    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()>;
    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize>;
    fn is_mapped(&self, address: u32, size: usize) -> bool;
}

#[allow(clippy::enum_variant_names)]
pub enum MemoryPermission {
    ReadExecute = 5,
    ReadWrite = 6,
    ReadWriteExecute = 7,
}

#[derive(Eq, PartialEq)]
pub enum ArmRegister {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    SB,
    SL,
    FP,
    IP,
    SP,
    LR,
    PC,
    Cpsr,
}
