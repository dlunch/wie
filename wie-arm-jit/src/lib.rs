#![no_std]
extern crate alloc;

use alloc::{string::String, vec::Vec};

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct RegionKey {
    pub pc: u32,
    pub thumb: bool,
    pub cpu_mode: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodePageStamp {
    pub page: u32,
    pub version: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CompiledHandle {
    pub slot: u32,
    pub generation: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RegionIr {
    pub entry: RegionKey,
    pub blocks: Vec<BasicBlock>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub pc: u32,
    pub size: u8,
    pub condition: Condition,
    pub operation: Operation,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Condition {
    Eq,
    Ne,
    Cs,
    Cc,
    Mi,
    Pl,
    Vs,
    Vc,
    Hi,
    Ls,
    Ge,
    Lt,
    Gt,
    Le,
    Always,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Register(u8),
    Immediate(u32),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Shift {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Rrx,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ShiftAmount {
    Immediate(u8),
    Register(u8),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Operand {
    pub value: Value,
    pub shift: Shift,
    pub amount: ShiftAmount,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum AluOp {
    And,
    Xor,
    Sub,
    ReverseSub,
    Add,
    AddCarry,
    SubCarry,
    ReverseSubCarry,
    Or,
    Move,
    BitClear,
    Not,
    Multiply,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Width {
    Byte,
    Half,
    Word,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub base: Value,
    pub offset: Operand,
    pub subtract: bool,
    pub pre_index: bool,
    pub write_back: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    Alu {
        op: AluOp,
        destination: Option<u8>,
        left: Value,
        right: Operand,
        set_flags: bool,
    },
    Branch {
        target: Value,
        link: Option<u32>,
        exchange: bool,
    },
    Load {
        destination: u8,
        address: Address,
        width: Width,
        signed: bool,
    },
    Store {
        value: Value,
        address: Address,
        width: Width,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompileRegion {
    pub ir: RegionIr,
    pub source: Vec<CodePageStamp>,
    pub expected_old: Option<CompiledHandle>,
}

impl CompileRegion {
    pub fn ir_size(&self) -> usize {
        core::mem::size_of::<Self>()
            + self.source.capacity() * core::mem::size_of::<CodePageStamp>()
            + self.ir.blocks.capacity() * core::mem::size_of::<BasicBlock>()
            + self
                .ir
                .blocks
                .iter()
                .map(|block| block.instructions.capacity() * core::mem::size_of::<Instruction>())
                .sum::<usize>()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CompileRequest {
    pub session: u64,
    pub request: u64,
    pub regions: Vec<CompileRegion>,
}

impl CompileRequest {
    pub fn ir_size(&self) -> usize {
        core::mem::size_of::<Self>()
            + (self.regions.capacity() - self.regions.len()) * core::mem::size_of::<CompileRegion>()
            + self.regions.iter().map(CompileRegion::ir_size).sum::<usize>()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ManifestRegion {
    pub entry: RegionKey,
    pub source: Vec<CodePageStamp>,
    pub export: String,
    pub expected_old: Option<CompiledHandle>,
}

pub struct CompiledRegion {
    pub manifest: ManifestRegion,
    pub handle: CompiledHandle,
}

pub struct CompiledArtifact {
    pub regions: Vec<CompiledRegion>,
    pub encoded_size: usize,
}

pub struct CompileCompletion {
    pub session: u64,
    pub request: u64,
    pub result: Result<CompiledArtifact, String>,
}

pub enum Admission {
    Accepted,
    Busy,
    Failed(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum CompiledExit {
    Dispatch = 0,
    Sample = 1,
    Budget = 2,
    End = 3,
    InterpretOne = 4,
    Invalidated = 5,
    GuestFault = 6,
}

#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct RunFrame {
    pub regs: [u32; 16],
    pub cpsr: u32,
    pub end: u32,
    pub budget_remaining: u32,
    pub sample_remaining: u32,
    pub executed: u32,
    pub fault_address: u32,
    pub scratch: u32,
}

pub enum AccessResult {
    Complete(u32),
    InterpretOne,
    Fault(u32),
    Invalidated,
}

pub trait ExecutionAccess {
    fn load(&mut self, address: u32, width: u32) -> AccessResult;
    fn store(&mut self, address: u32, width: u32, value: u32) -> AccessResult;
    fn sample_prepare(&mut self, pc: u32, cpsr: u32, r7: u32);
}

pub trait CompiledExecutor: Send {
    fn submit(&mut self, request: CompileRequest) -> Admission;
    fn poll(&mut self) -> Option<CompileCompletion>;
    fn execute(&mut self, handle: CompiledHandle, frame: &mut RunFrame, access: &mut dyn ExecutionAccess) -> Result<CompiledExit, String>;
    fn retire(&mut self, handles: &[CompiledHandle]);
    fn shutdown(&mut self);
}

#[cfg(test)]
mod tests {
    use core::mem::{align_of, offset_of, size_of};

    use super::RunFrame;

    #[test]
    fn generated_code_frame_has_a_fixed_plain_data_layout() {
        assert_eq!(size_of::<RunFrame>(), 92);
        assert_eq!(align_of::<RunFrame>(), 4);
        assert_eq!(offset_of!(RunFrame, regs), 0);
        assert_eq!(offset_of!(RunFrame, cpsr), 64);
        assert_eq!(offset_of!(RunFrame, end), 68);
        assert_eq!(offset_of!(RunFrame, budget_remaining), 72);
        assert_eq!(offset_of!(RunFrame, sample_remaining), 76);
        assert_eq!(offset_of!(RunFrame, executed), 80);
        assert_eq!(offset_of!(RunFrame, fault_address), 84);
        assert_eq!(offset_of!(RunFrame, scratch), 88);
        assert_eq!(bytemuck::bytes_of(&RunFrame::default()), &[0; 92]);
    }
}
