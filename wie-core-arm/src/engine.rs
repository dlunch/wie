mod arm32_cpu;
mod debugged_arm32_cpu;
mod sampler;

use alloc::vec::Vec;

use web_time::Instant;

use wie_backend::ProfileSample;
use wie_util::{AsAny, Result};

pub use arm32_cpu::Arm32CpuEngine;
pub(crate) use arm32_cpu::EmulatedMemory;
pub use debugged_arm32_cpu::DebuggedArm32CpuEngine;
pub(crate) use debugged_arm32_cpu::{DebugBreakpointKind, DebugInner, DebugSignal, DebugStopReason};

pub enum EngineStopReason {
    End,
    Yield,
    Deadline,
    Svc { category: u32, lr: u32, spsr: u32 },
}

pub struct EngineRunResult {
    pub stop_reason: EngineStopReason,
    pub instructions_executed: u32,
}

pub trait ArmEngine: Send + AsAny {
    fn run(&mut self, end: u32, count: u32, deadline: Option<Instant>) -> Result<EngineRunResult>;
    fn mark_entry(&mut self);
    fn reg_write(&mut self, reg: ArmRegister, value: u32);
    fn reg_read(&self, reg: ArmRegister) -> u32;
    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission);
    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()>;
    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize>;
    fn is_mapped(&self, address: u32, size: usize) -> bool;
    fn set_profiling(&mut self, enabled: bool);
    fn take_profile(&mut self, force: bool) -> Vec<ProfileSample>;
    fn maintain(&mut self);
    fn shutdown(&mut self);
}

#[allow(clippy::enum_variant_names)]
pub enum MemoryPermission {
    ReadExecute = 5,
    ReadWrite = 6,
    ReadWriteExecute = 7,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
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
