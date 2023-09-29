mod unicorn;

use alloc::vec::Vec;
use core::ops::Range;

pub use unicorn::UnicornEngine;

pub type ArmEngineResult<T> = anyhow::Result<T>;
pub type ArmEngineError = anyhow::Error;

pub trait ArmEngine {
    fn run(&mut self, end: u32, hook: Range<u32>) -> ArmEngineResult<()>;
    fn reg_write(&mut self, reg: ArmRegister, value: u32);
    fn reg_read(&self, reg: ArmRegister) -> u32;
    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission);
    fn mem_write(&mut self, address: u32, data: &[u8]) -> ArmEngineResult<()>;
    fn mem_read(&self, address: u32, size: usize) -> ArmEngineResult<Vec<u8>>;
}

#[allow(clippy::enum_variant_names)]
pub enum MemoryPermission {
    ReadExecute = 5,
    ReadWrite = 6,
    ReadWriteExecute = 7,
}

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
