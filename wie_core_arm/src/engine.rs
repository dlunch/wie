mod arm32_cpu;
#[cfg(not(target_arch = "wasm32"))]
mod debugged_arm32_cpu;

use alloc::vec::Vec;

use wie_util::{AsAny, Result};

use crate::context::ArmCoreContext;

pub use arm32_cpu::Arm32CpuEngine;
#[cfg(not(target_arch = "wasm32"))]
pub use debugged_arm32_cpu::DebuggedArm32CpuEngine;
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use debugged_arm32_cpu::{DebugBreakpointKind, DebugInner, DebugSignal, DebugStopReason};

pub trait ArmEngine: Send + AsAny {
    fn run(&mut self, end: u32, count: u32) -> Result<EngineRunResult>;
    fn reg_write(&mut self, reg: ArmRegister, value: u32);
    fn reg_read(&self, reg: ArmRegister) -> u32;
    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission);
    fn mem_write(&mut self, address: u32, data: &[u8]) -> Result<()>;
    fn mem_read(&mut self, address: u32, size: usize, result: &mut [u8]) -> Result<usize>;
    fn is_mapped(&self, address: u32, size: usize) -> bool;
    fn save_state(&self) -> ArmEngineState;
    fn restore_state(&mut self, state: &ArmEngineState) -> Result<()>;
}

#[derive(Clone)]
pub struct ArmEngineState {
    pub context: ArmCoreContext,
    pub pages: Vec<(u32, Vec<u8>)>,
}

pub enum EngineRunResult {
    ReachedEnd { pc: u32 },
    CountExpired { pc: u32 },
    Svc { pc: u32, immediate: u32, r12: u32, lr: u32, spsr: u32 },
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
