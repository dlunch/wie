use alloc::{vec, vec::Vec};

use super::{AluOp, BasicBlock, BranchTarget, Condition, MemoryOperand, Operation, Reg, Shift, ShiftAmount, Value};

impl MemoryOperand {
    /// Effective address and indexed base, with the guest's 32-bit wrapping arithmetic.
    pub fn constant_address(&self) -> Option<(u32, u32)> {
        let Value::Immediate(base) = self.base else { return None };
        let (offset, _) = self.offset.constant_value()?;
        let indexed = if self.subtract {
            base.wrapping_sub(offset)
        } else {
            base.wrapping_add(offset)
        };
        Some((if self.pre_index { indexed } else { base }, indexed))
    }
}

impl AluOp {
    /// Fold the result when the operands determine it without the incoming CPSR.
    pub fn constant_result(&self, left: &Value, right: u32) -> Option<u32> {
        match self {
            Self::Move => return Some(right),
            Self::Not => return Some(!right),
            Self::CountLeadingZeros => return Some(right.leading_zeros()),
            _ => {}
        }
        let Value::Immediate(left) = left else { return None };
        Some(match self {
            Self::Add => left.wrapping_add(right),
            Self::Sub => left.wrapping_sub(right),
            Self::ReverseSub => right.wrapping_sub(*left),
            Self::And => left & right,
            Self::Or => left | right,
            Self::Xor => left ^ right,
            Self::BitClear => left & !right,
            Self::Multiply => left.wrapping_mul(right),
            _ => return None,
        })
    }
}

impl Condition {
    pub fn flag_inputs(&self) -> u8 {
        match self {
            Condition::Eq | Condition::Ne => 0x4,
            Condition::Cs | Condition::Cc => 0x2,
            Condition::Mi | Condition::Pl => 0x8,
            Condition::Vs | Condition::Vc => 0x1,
            Condition::Hi | Condition::Ls => 0x6,
            Condition::Ge | Condition::Lt => 0x9,
            Condition::Gt | Condition::Le => 0xd,
            Condition::Always => 0,
        }
    }
}

impl Operation {
    /// NZCV bits implicitly produced, unconditionally replaced when executed, and consumed.
    /// Explicit CPSR reads and writes operate on the whole register separately.
    pub fn implicit_flag_effects(&self) -> (u8, u8, u8) {
        match self {
            Operation::Alu { op, right, set_flags, .. } => {
                let carry_input = matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) || right.shift == Shift::Rrx;
                let (written, killed) = if !set_flags {
                    (0, 0)
                } else if matches!(
                    op,
                    AluOp::Add | AluOp::Sub | AluOp::ReverseSub | AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry
                ) {
                    (0xf, 0xf)
                } else if matches!(op, AluOp::Multiply | AluOp::CountLeadingZeros) {
                    (0xc, 0xc)
                } else if right.shift == Shift::Rrx {
                    (0xe, 0xe)
                } else {
                    match right.amount {
                        ShiftAmount::Immediate(0) => (0xc, 0xc),
                        ShiftAmount::Immediate(_) => (0xe, 0xe),
                        // A zero register shift preserves carry instead of overwriting it.
                        ShiftAmount::Register(_) => (0xe, 0xc),
                    }
                };
                (written, killed, if carry_input { 0x2 } else { 0 })
            }
            Operation::MultiplyAccumulate { set_flags, .. } | Operation::MultiplyLong { set_flags, .. } => {
                let written = if *set_flags { 0xc } else { 0 };
                (written, written, 0)
            }
            Operation::Load { address, .. } | Operation::Store { address, .. } | Operation::DoubleTransfer { address, .. } => {
                (0, 0, if address.offset.shift == Shift::Rrx { 0x2 } else { 0 })
            }
            Self::MultipleTransfer { .. } | Self::Swap { .. } | Self::Nop | Self::Branch { .. } | Self::ReadCpsr { .. } | Self::WriteCpsr { .. } => {
                (0, 0, 0)
            }
        }
    }

    /// Visit each general-purpose register operand, including writeback and transfers.
    pub fn visit_registers(&self, mut read: impl FnMut(&Reg), mut write: impl FnMut(&Reg)) {
        let mut input = None;
        let mut operand = None;
        match self {
            Operation::Alu {
                op,
                destination,
                left,
                right,
                ..
            } => {
                if !matches!(op, AluOp::Move | AluOp::Not | AluOp::CountLeadingZeros) {
                    input = Some(left);
                }
                operand = Some(right);
                if let Some(reg) = destination {
                    write(reg);
                }
            }
            Operation::Branch { target, link, .. } => {
                write(&Reg::PC);
                if let BranchTarget::Register(reg) = target {
                    read(reg);
                }
                if link.is_some() {
                    write(&Reg::LR);
                }
            }
            Operation::Load { destination, .. } => write(destination),
            Operation::Store { value, .. } => input = Some(value),
            Operation::DoubleTransfer { register, load, .. } => {
                for reg in [register, &Reg::new(register.index() + 1)] {
                    if *load { write(reg) } else { read(reg) }
                }
            }
            Operation::MultipleTransfer {
                base,
                registers,
                load,
                write_back,
                ..
            } => {
                read(base);
                if *write_back {
                    write(base);
                }
                for reg in 0..16 {
                    if registers & (1 << reg) != 0 {
                        if *load { write(&Reg::new(reg)) } else { read(&Reg::new(reg)) }
                    }
                }
            }
            Operation::Swap {
                destination, address, value, ..
            } => {
                read(address);
                read(value);
                write(destination);
            }
            Operation::MultiplyAccumulate {
                destination,
                left,
                right,
                accumulate,
                ..
            } => {
                for reg in [left, right, accumulate] {
                    read(reg);
                }
                write(destination);
            }
            Operation::MultiplyLong {
                low,
                high,
                left,
                right,
                accumulate,
                ..
            } => {
                read(left);
                read(right);
                for reg in [low, high] {
                    write(reg);
                    if *accumulate {
                        read(reg);
                    }
                }
            }
            Operation::ReadCpsr { destination } => write(destination),
            Operation::WriteCpsr { value, .. } => input = Some(value),
            Operation::Nop => {}
        }
        if let Self::Load { address, .. } | Self::Store { address, .. } | Self::DoubleTransfer { address, .. } = self {
            if let Value::Register(reg) = &address.base {
                read(reg);
            }
            if let Some(reg) = &address.write_back {
                write(reg);
            }
            operand = Some(&address.offset);
        }
        if let Some(operand) = operand {
            if let Value::Register(reg) = &operand.value {
                read(reg);
            }
            if let ShiftAmount::Register(reg) = &operand.amount {
                read(reg);
            }
        }
        if let Some(Value::Register(reg)) = input {
            read(reg);
        }
    }
}

impl BasicBlock {
    /// Returns the live NZCV output bits (8, 4, 2, 1) of each instruction.
    pub fn live_flag_updates(&self) -> Vec<u8> {
        let mut updates = vec![0; self.instructions.len()];
        let mut live = 0xf;
        let mut next_pc = None;
        for (instruction, update) in self.instructions.iter().zip(&mut updates).rev() {
            if next_pc != Some(instruction.pc.get().wrapping_add(u32::from(instruction.size))) || instruction.operation.writes_pc() {
                live = 0xf;
            }
            next_pc = Some(instruction.pc.get());
            if matches!(
                instruction.operation,
                Operation::Branch { .. } | Operation::ReadCpsr { .. } | Operation::WriteCpsr { .. }
            ) {
                live = 0xf;
            }
            let (written, killed, read) = instruction.operation.implicit_flag_effects();
            *update = written & live;
            if instruction.condition == Condition::Always {
                live &= !killed;
            }
            live |= read | instruction.condition.flag_inputs();
        }
        updates
    }

    /// Suppress ALU stores whose value is unchanged or overwritten before it is observable.
    /// Flag effects remain attached to the original instruction.
    pub fn eliminate_dead_alu_stores(&mut self) {
        let mut live = u16::MAX;
        let mut next_pc = None;
        for ins in self.instructions.iter_mut().rev() {
            if next_pc != Some(ins.pc.get().wrapping_add(u32::from(ins.size))) || ins.operation.writes_pc() {
                live = u16::MAX;
            }
            next_pc = Some(ins.pc.get());
            let Operation::Alu {
                op,
                destination,
                left,
                right,
                ..
            } = &mut ins.operation
            else {
                live = u16::MAX;
                continue;
            };
            if let Some(rd) = destination.as_ref() {
                let unshifted = right.amount == ShiftAmount::Immediate(0) && right.shift != Shift::Rrx;
                let same_right = unshifted && matches!(&right.value, Value::Register(r) if r == rd);
                let same_left = matches!(left, Value::Register(r) if r == rd);
                let redundant = *rd != Reg::PC
                    && (same_right && (*op == AluOp::Move || (same_left && matches!(op, AluOp::And | AluOp::Or)))
                        || (same_left
                            && matches!(op, AluOp::Add | AluOp::Sub | AluOp::Or | AluOp::Xor | AluOp::BitClear)
                            && right.constant_value().is_some_and(|(value, _)| value == 0)));
                if redundant {
                    *destination = None;
                }
            }
            if let Some(rd) = destination {
                let bit = 1 << rd.index();
                if *rd != Reg::PC && live & bit == 0 {
                    *destination = None;
                } else if ins.condition == Condition::Always {
                    live &= !bit;
                }
            }
            ins.operation.visit_registers(|reg| live |= 1 << reg.index(), |_| {});
        }
    }
}
