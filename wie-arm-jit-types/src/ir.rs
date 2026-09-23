use alloc::vec::Vec;

use bytemuck::Contiguous;

use crate::RegionKey;

#[derive(Clone, Copy, PartialEq)]
pub struct Reg(u8);

impl Reg {
    pub const SP: Self = Self(13);
    pub const LR: Self = Self(14);
    pub const PC: Self = Self(15);

    pub const fn new(index: u8) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u8 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct MemoryAddress(u32);

impl MemoryAddress {
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

pub enum BranchTarget {
    Register(Reg),
    Address(MemoryAddress),
}

pub struct RegionIr {
    pub entry: RegionKey,
    pub blocks: Vec<BasicBlock>,
}

pub struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

pub struct Instruction {
    pub pc: MemoryAddress,
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
    Register(Reg),
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
    Register(Reg),
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

pub struct MemoryOperand {
    pub base: Value,
    pub offset: Operand,
    pub subtract: bool,
    pub pre_index: bool,
    pub write_back: Option<Reg>,
}

pub enum Operation {
    Alu {
        op: AluOp,
        destination: Option<Reg>,
        left: Value,
        right: Operand,
        set_flags: bool,
    },
    Branch {
        target: BranchTarget,
        link: Option<MemoryAddress>,
        exchange: bool,
    },
    Load {
        destination: Reg,
        address: MemoryOperand,
        width: Width,
        signed: bool,
    },
    Store {
        value: Value,
        address: MemoryOperand,
        width: Width,
    },
    MultiplyAccumulate {
        destination: Reg,
        left: Reg,
        right: Reg,
        accumulate: Reg,
        set_flags: bool,
    },
    MultiplyLong {
        low: Reg,
        high: Reg,
        left: Reg,
        right: Reg,
        signed: bool,
        accumulate: bool,
        set_flags: bool,
    },
    ReadCpsr {
        destination: Reg,
    },
    WriteCpsr {
        value: Value,
        mask: u32,
    },
    MultipleTransfer {
        base: Reg,
        registers: u16,
        increment: bool,
        before: bool,
        write_back: bool,
        load: bool,
    },
    DoubleTransfer {
        register: Reg,
        address: MemoryOperand,
        load: bool,
    },
    Swap {
        destination: Reg,
        address: Reg,
        value: Reg,
        width: Width,
    },
    Nop,
}

impl Operation {
    pub fn writes_pc(&self) -> bool {
        match self {
            Self::Branch { .. }
            | Self::Alu {
                destination: Some(Reg::PC), ..
            }
            | Self::Load { destination: Reg::PC, .. } => true,
            Self::MultipleTransfer { registers, load: true, .. } => registers & (1 << Reg::PC.index()) != 0,
            _ => false,
        }
    }
}
