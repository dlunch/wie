use alloc::vec::Vec;

use bytemuck::Contiguous;

use crate::RegionKey;

pub struct RegionIr {
    pub entry: RegionKey,
    pub blocks: Vec<BasicBlock>,
}

pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

pub struct Instruction {
    pub pc: u32,
    pub size: u8,
    pub condition: Condition,
    pub operation: Operation,
}

/// ARM condition-field encodings; 0xf belongs to the unconditional instruction space.
#[derive(Clone, Copy, PartialEq, Contiguous)]
#[repr(u8)]
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

#[derive(Clone, Copy, PartialEq)]
pub enum Value {
    Register(u8),
    Immediate(u32),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Shift {
    Lsl,
    Lsr,
    Asr,
    Ror,
    Rrx,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ShiftAmount {
    Immediate(u8),
    Register(u8),
}

pub struct Operand {
    pub value: Value,
    pub shift: Shift,
    pub amount: ShiftAmount,
}

#[derive(Clone, Copy, PartialEq)]
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
    CountLeadingZeros,
}

pub enum Width {
    Byte,
    Half,
    Word,
}

pub struct Address {
    pub base: Value,
    pub offset: Operand,
    pub subtract: bool,
    pub pre_index: bool,
    pub write_back: Option<u8>,
}

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
    MultiplyAccumulate {
        destination: u8,
        left: u8,
        right: u8,
        accumulate: u8,
        set_flags: bool,
    },
    MultiplyLong {
        low: u8,
        high: u8,
        left: u8,
        right: u8,
        signed: bool,
        accumulate: bool,
        set_flags: bool,
    },
    ReadStatus {
        destination: u8,
    },
    WriteStatus {
        value: Value,
        mask: u32,
    },
    MultipleTransfer {
        base: u8,
        registers: u16,
        increment: bool,
        before: bool,
        write_back: bool,
        load: bool,
    },
    DoubleTransfer {
        register: u8,
        address: Address,
        load: bool,
    },
    Swap {
        destination: u8,
        address: u8,
        value: u8,
        width: Width,
    },
    Nop,
}

impl Operation {
    pub fn writes_pc(&self) -> bool {
        match self {
            Self::Branch { .. } | Self::Alu { destination: Some(15), .. } | Self::Load { destination: 15, .. } => true,
            Self::MultipleTransfer { registers, load: true, .. } => registers & 0x8000 != 0,
            _ => false,
        }
    }
}
