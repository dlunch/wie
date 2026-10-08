use std::{
    collections::BTreeMap,
    mem::{offset_of, size_of},
};

use cranelift_codegen::{
    Context,
    ir::{Block, InstBuilder, MemFlagsData, Type, Value, condcodes::IntCC, types},
    isa::TargetIsa,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch};

use wie_arm_jit_types::{
    CompiledExit, RunFrame,
    ir::{self, AluOp, BranchTarget, Condition, Instruction, MemoryOperand, Operand, Operation, Reg, RegionIr, Shift, ShiftAmount},
};
use wie_util::{Result, WieError};

mod memory;

pub(crate) fn emit_region(ir: &RegionIr, context: &mut Context, isa: &dyn TargetIsa) -> Result<()> {
    // The IR is public input: reject out-of-frame registers before creating native accesses.
    let register = |register: &Reg| {
        if register.index() > 15 {
            Err(WieError::FatalError(format!("Native ARM register {} is out of range", register.index())))
        } else {
            Ok(())
        }
    };
    let value = |value: &ir::Value| -> Result<()> {
        if let ir::Value::Register(reg) = value {
            register(reg)?;
        }
        Ok(())
    };
    let operand = |operand: &Operand| -> Result<()> {
        value(&operand.value)?;
        if let ShiftAmount::Register(reg) = &operand.amount {
            register(reg)?;
        }
        Ok(())
    };
    let address = |address: &MemoryOperand| -> Result<()> {
        value(&address.base)?;
        operand(&address.offset)?;
        if let Some(reg) = &address.write_back {
            register(reg)?;
        }
        Ok(())
    };
    for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
        match &instruction.operation {
            Operation::Alu {
                destination, left, right, ..
            } => {
                if let Some(destination) = destination {
                    register(destination)?;
                }
                value(left)?;
                operand(right)?;
            }
            Operation::Branch { target, .. } => {
                if let BranchTarget::Register(reg) = target {
                    register(reg)?;
                }
            }
            Operation::Load {
                destination,
                address: memory,
                ..
            } => {
                register(destination)?;
                address(memory)?;
            }
            Operation::Store {
                value: source,
                address: memory,
                ..
            } => {
                value(source)?;
                address(memory)?;
            }
            Operation::MultiplyAccumulate {
                destination,
                left,
                right,
                accumulate,
                ..
            } => {
                for reg in [destination, left, right, accumulate] {
                    register(reg)?;
                }
            }
            Operation::MultiplyLong { low, high, left, right, .. } => {
                for reg in [low, high, left, right] {
                    register(reg)?;
                }
            }
            Operation::ReadCpsr { destination } => register(destination)?,
            Operation::WriteCpsr { value: source, .. } => value(source)?,
            Operation::MultipleTransfer { base, .. } => register(base)?,
            Operation::DoubleTransfer {
                register: first,
                address: memory,
                ..
            } => {
                register(first)?;
                register(&Reg::new(first.index() + 1))?;
                address(memory)?;
            }
            Operation::Swap {
                destination, address, value, ..
            } => {
                for reg in [destination, address, value] {
                    register(reg)?;
                }
            }
            Operation::Nop => {}
        }
    }
    let mut builder_context = FunctionBuilderContext::new();
    let mut builder = FunctionBuilder::new(&mut context.func, &mut builder_context);
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    let frame = builder.block_params(entry)[0];
    let pages = builder.block_params(entry)[1];
    let dispatch = builder.create_block();
    let end = builder.create_block();
    let interpret = builder.create_block();
    let leave = builder.create_block();
    let entries: BTreeMap<_, _> = ir
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .map(|instruction| (instruction.pc.get(), builder.create_block()))
        .collect();
    let mut emitter = Emitter {
        builder,
        frame,
        pages,
        ptr_type: isa.pointer_type(),
        thumb: ir.entry.thumb,
        pc: ir.entry.pc,
        mode: u32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 },
        entries,
        dispatch,
        end,
        interpret,
        leave,
    };
    let pc = emitter.load_frame(offset_of!(RunFrame, regs) + 15 * size_of::<u32>());
    emitter.boundaries(pc);
    let mut switch = Switch::new();
    for (&pc, &block) in &emitter.entries {
        switch.set_entry(u128::from(pc), block);
    }
    switch.emit(&mut emitter.builder, pc, dispatch);
    for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
        emitter.pc = instruction.pc.get();
        emitter.builder.switch_to_block(emitter.entries[&emitter.pc]);
        let pc = emitter.builder.ins().iconst(types::I32, i64::from(emitter.pc));
        emitter.boundaries(pc);
        let fallthrough = emitter.pc.wrapping_add(u32::from(instruction.size));
        if instruction.condition != Condition::Always {
            let condition = emitter.condition(&instruction.condition);
            let execute = emitter.builder.create_block();
            let skip = emitter.builder.create_block();
            emitter.builder.ins().brif(condition, execute, &[], skip, &[]);
            emitter.builder.switch_to_block(skip);
            let next = emitter.builder.ins().iconst(types::I32, i64::from(fallthrough));
            emitter.commit(next, Some(fallthrough));
            emitter.builder.switch_to_block(execute);
        }
        let next = emitter.operation(instruction);
        let fixed = match &instruction.operation {
            Operation::Branch {
                target: BranchTarget::Address(address),
                exchange,
                ..
            } if !exchange || (address.get() & 1 != 0) == emitter.thumb => Some(address.get() & if emitter.thumb { !1 } else { !3 }),
            _ if next.is_none() => Some(fallthrough),
            _ => None,
        };
        let next = next.unwrap_or_else(|| emitter.builder.ins().iconst(types::I32, i64::from(fallthrough)));
        emitter.commit(next, fixed);
    }
    emitter.builder.switch_to_block(leave);
    let pc = emitter.load_frame(offset_of!(RunFrame, regs) + 15 * size_of::<u32>());
    emitter.boundaries(pc);
    emitter.builder.ins().jump(dispatch, &[]);
    for (block, exit) in [
        (dispatch, CompiledExit::Dispatch),
        (end, CompiledExit::End),
        (interpret, CompiledExit::InterpretOne),
    ] {
        emitter.builder.switch_to_block(block);
        emitter.exit(exit);
    }
    emitter.builder.seal_all_blocks();
    emitter.builder.finalize(isa.frontend_config());
    Ok(())
}

struct Emitter<'a> {
    builder: FunctionBuilder<'a>,
    frame: Value,
    pages: Value,
    ptr_type: Type,
    thumb: bool,
    pc: u32,
    mode: u32,
    entries: BTreeMap<u32, Block>,
    dispatch: Block,
    end: Block,
    interpret: Block,
    leave: Block,
}

impl Emitter<'_> {
    fn load_frame(&mut self, offset: usize) -> Value {
        self.builder.ins().load(types::I32, MemFlagsData::trusted(), self.frame, offset as i32)
    }

    fn store_frame(&mut self, offset: usize, value: Value) {
        self.builder.ins().store(MemFlagsData::trusted(), value, self.frame, offset as i32);
    }

    fn register(&mut self, register: &Reg) -> Value {
        if *register == Reg::PC {
            self.builder
                .ins()
                .iconst(types::I32, i64::from(self.pc.wrapping_add(if self.thumb { 4 } else { 8 })))
        } else {
            self.load_frame(offset_of!(RunFrame, regs) + usize::from(register.index()) * size_of::<u32>())
        }
    }

    fn store_register(&mut self, register: &Reg, value: Value) {
        self.store_frame(offset_of!(RunFrame, regs) + usize::from(register.index()) * size_of::<u32>(), value);
    }

    fn value(&mut self, value: &ir::Value) -> Value {
        match value {
            ir::Value::Register(register) => self.register(register),
            ir::Value::Immediate(value) => self.builder.ins().iconst(types::I32, i64::from(*value)),
        }
    }

    fn cpsr(&mut self) -> Value {
        self.load_frame(offset_of!(RunFrame, cpsr))
    }

    fn guard(&mut self, condition: Value, exit: CompiledExit) {
        let target = match exit {
            CompiledExit::Dispatch => self.dispatch,
            CompiledExit::End => self.end,
            CompiledExit::InterpretOne => self.interpret,
            CompiledExit::GuestFault => unreachable!(),
        };
        let next = self.builder.create_block();
        self.builder.ins().brif(condition, target, &[], next, &[]);
        self.builder.switch_to_block(next);
    }

    fn exit(&mut self, exit: CompiledExit) {
        let value = self.builder.ins().iconst(types::I32, exit as i64);
        self.builder.ins().return_(&[value]);
    }

    fn boundaries(&mut self, pc: Value) {
        let low = self.builder.ins().icmp_imm_s(IntCC::UnsignedLessThan, pc, 0x1000);
        self.guard(low, CompiledExit::Dispatch);
        let cpsr = self.cpsr();
        let mode = self.builder.ins().band_imm_s(cpsr, 0x0100_003f);
        let changed = self.builder.ins().icmp_imm_s(IntCC::NotEqual, mode, i64::from(self.mode));
        self.guard(changed, CompiledExit::Dispatch);
        let end = self.load_frame(offset_of!(RunFrame, end));
        let at_end = self.builder.ins().icmp(IntCC::Equal, pc, end);
        self.guard(at_end, CompiledExit::End);
        let executed = self.load_frame(offset_of!(RunFrame, executed));
        let budget = self.load_frame(offset_of!(RunFrame, budget));
        let exhausted = self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, executed, budget);
        self.guard(exhausted, CompiledExit::Dispatch);
    }

    fn commit(&mut self, next: Value, fixed: Option<u32>) {
        self.store_register(&Reg::PC, next);
        let executed = self.load_frame(offset_of!(RunFrame, executed));
        let executed = self.builder.ins().iadd_imm_s(executed, 1);
        self.store_frame(offset_of!(RunFrame, executed), executed);
        let target = fixed.and_then(|pc| self.entries.get(&pc)).copied().unwrap_or(self.leave);
        self.builder.ins().jump(target, &[]);
    }

    fn pc_write(&mut self, value: Value, exchange: bool) -> Value {
        if exchange {
            let thumb = self.builder.ins().band_imm_s(value, 1);
            let bit = self.builder.ins().ishl_imm_s(thumb, 5);
            let cpsr = self.cpsr();
            let cpsr = self.builder.ins().band_imm_s(cpsr, !0x20);
            let cpsr = self.builder.ins().bor(cpsr, bit);
            self.store_frame(offset_of!(RunFrame, cpsr), cpsr);
            let thumb_mask = self.builder.ins().iconst(types::I32, -2);
            let arm_mask = self.builder.ins().iconst(types::I32, -4);
            let mask = self.builder.ins().select(thumb, thumb_mask, arm_mask);
            self.builder.ins().band(value, mask)
        } else {
            self.builder.ins().band_imm_s(value, if self.thumb { -2 } else { -4 })
        }
    }

    fn operand(&mut self, operand: &Operand) -> (Value, Value) {
        let value = self.value(&operand.value);
        let cpsr = self.cpsr();
        let carry = self.builder.ins().ushr_imm_s(cpsr, 29);
        let carry = self.builder.ins().band_imm_s(carry, 1);
        if operand.shift == Shift::Rrx {
            let high = self.builder.ins().ishl_imm_s(carry, 31);
            let low = self.builder.ins().ushr_imm_s(value, 1);
            let result = self.builder.ins().bor(high, low);
            let carry = self.builder.ins().band_imm_s(value, 1);
            return (result, carry);
        }
        let amount = match &operand.amount {
            ShiftAmount::Immediate(0) => return (value, carry),
            ShiftAmount::Immediate(amount) => self.builder.ins().iconst(types::I32, i64::from(*amount)),
            ShiftAmount::Register(register) => {
                let amount = self.register(register);
                self.builder.ins().band_imm_s(amount, 255)
            }
        };
        let zero = self.builder.ins().iconst(types::I32, 0);
        let (result, shifted_carry) = match operand.shift {
            Shift::Lsl | Shift::Lsr => {
                // Host shifts mask their counts; ARM instead distinguishes 32 and larger counts.
                let within_word = self.builder.ins().icmp_imm_s(IntCC::UnsignedLessThan, amount, 32);
                let shifted = if operand.shift == Shift::Lsl {
                    self.builder.ins().ishl(value, amount)
                } else {
                    self.builder.ins().ushr(value, amount)
                };
                let result = self.builder.ins().select(within_word, shifted, zero);
                let carry_amount = if operand.shift == Shift::Lsl {
                    let bits = self.builder.ins().iconst(types::I32, 32);
                    self.builder.ins().isub(bits, amount)
                } else {
                    self.builder.ins().iadd_imm_s(amount, -1)
                };
                let carry_bit = self.builder.ins().ushr(value, carry_amount);
                let carry_bit = self.builder.ins().band_imm_s(carry_bit, 1);
                let has_carry = self.builder.ins().icmp_imm_s(IntCC::UnsignedLessThanOrEqual, amount, 32);
                let shifted_carry = self.builder.ins().select(has_carry, carry_bit, zero);
                (result, shifted_carry)
            }
            Shift::Asr => {
                let sign_bit = self.builder.ins().iconst(types::I32, 31);
                let large = self.builder.ins().icmp_imm_s(IntCC::UnsignedGreaterThan, amount, 31);
                let result_amount = self.builder.ins().select(large, sign_bit, amount);
                let result = self.builder.ins().sshr(value, result_amount);
                let previous = self.builder.ins().iadd_imm_s(amount, -1);
                let carry_amount = self.builder.ins().select(large, sign_bit, previous);
                let shifted_carry = self.builder.ins().ushr(value, carry_amount);
                let shifted_carry = self.builder.ins().band_imm_s(shifted_carry, 1);
                (result, shifted_carry)
            }
            Shift::Ror => {
                let result = self.builder.ins().rotr(value, amount);
                let shifted_carry = self.builder.ins().ushr_imm_s(result, 31);
                (result, shifted_carry)
            }
            Shift::Rrx => unreachable!(),
        };
        let unchanged = self.builder.ins().icmp_imm_s(IntCC::Equal, amount, 0);
        let result = self.builder.ins().select(unchanged, value, result);
        let carry = self.builder.ins().select(unchanged, carry, shifted_carry);
        (result, carry)
    }

    fn set_flags(&mut self, result: Value, carry: Option<Value>, overflow: Option<Value>) {
        let high = if self.builder.func.dfg.value_type(result) == types::I64 {
            let high = self.builder.ins().ushr_imm_s(result, 32);
            self.builder.ins().ireduce(types::I32, high)
        } else {
            result
        };
        let negative = self.builder.ins().band_imm_s(high, 0x8000_0000);
        let zero = self.builder.ins().icmp_imm_s(IntCC::Equal, result, 0);
        let zero = self.builder.ins().uextend(types::I32, zero);
        let zero = self.builder.ins().ishl_imm_s(zero, 30);
        let mut flags = self.builder.ins().bor(negative, zero);
        let mut mask = 0xc000_0000_u32;
        for (value, bit) in [(carry, 29), (overflow, 28)] {
            if let Some(value) = value {
                let value = self.builder.ins().ishl_imm_s(value, bit);
                flags = self.builder.ins().bor(flags, value);
                mask |= 1 << bit;
            }
        }
        let cpsr = self.cpsr();
        let preserved = self.builder.ins().band_imm_s(cpsr, i64::from(!mask));
        let cpsr = self.builder.ins().bor(preserved, flags);
        self.store_frame(offset_of!(RunFrame, cpsr), cpsr);
    }

    fn condition(&mut self, condition: &Condition) -> Value {
        let cpsr = self.cpsr();
        let result = match condition {
            Condition::Eq | Condition::Ne | Condition::Cs | Condition::Cc | Condition::Mi | Condition::Pl | Condition::Vs | Condition::Vc => {
                let bit = match condition {
                    Condition::Eq | Condition::Ne => 30,
                    Condition::Cs | Condition::Cc => 29,
                    Condition::Vs | Condition::Vc => 28,
                    _ => 31,
                };
                let bit = self.builder.ins().band_imm_s(cpsr, 1_i64 << bit);
                self.builder.ins().icmp_imm_s(IntCC::NotEqual, bit, 0)
            }
            Condition::Hi | Condition::Ls => {
                let flags = self.builder.ins().band_imm_s(cpsr, 0x6000_0000);
                self.builder.ins().icmp_imm_s(IntCC::Equal, flags, 0x2000_0000)
            }
            Condition::Ge | Condition::Lt | Condition::Gt | Condition::Le => {
                let shifted = self.builder.ins().ishl_imm_s(cpsr, 3);
                let mut flags = self.builder.ins().bxor(cpsr, shifted);
                if matches!(condition, Condition::Gt | Condition::Le) {
                    let zero = self.builder.ins().ishl_imm_s(cpsr, 1);
                    flags = self.builder.ins().bor(flags, zero);
                }
                self.builder.ins().icmp_imm_s(IntCC::SignedGreaterThanOrEqual, flags, 0)
            }
            Condition::Always => self.builder.ins().iconst(types::I8, 1),
        };
        if matches!(
            condition,
            Condition::Ne | Condition::Cc | Condition::Pl | Condition::Vc | Condition::Ls | Condition::Lt | Condition::Le
        ) {
            self.builder.ins().icmp_imm_s(IntCC::Equal, result, 0)
        } else {
            result
        }
    }

    fn operation(&mut self, instruction: &Instruction) -> Option<Value> {
        match &instruction.operation {
            Operation::Alu {
                op,
                destination,
                left,
                right,
                set_flags,
            } => {
                let mut left = self.value(left);
                let (mut right, shifted_carry) = self.operand(right);
                let mut carry = Some(shifted_carry);
                let mut overflow = None;
                let result = match op {
                    AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry => {
                        if matches!(op, AluOp::ReverseSub | AluOp::ReverseSubCarry) {
                            std::mem::swap(&mut left, &mut right);
                        }
                        let subtract = !matches!(op, AluOp::Add | AluOp::AddCarry);
                        if subtract {
                            right = self.builder.ins().bnot(right);
                        }
                        let carry_in = if matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) {
                            let cpsr = self.cpsr();
                            let carry = self.builder.ins().ushr_imm_s(cpsr, 29);
                            self.builder.ins().band_imm_s(carry, 1)
                        } else {
                            self.builder.ins().iconst(types::I32, i64::from(subtract))
                        };
                        if *set_flags {
                            // Complementing the subtrahend gives ARM's no-borrow carry bit.
                            let wide_left = self.builder.ins().uextend(types::I64, left);
                            let wide_right = self.builder.ins().uextend(types::I64, right);
                            let wide_carry = self.builder.ins().uextend(types::I64, carry_in);
                            let wide = self.builder.ins().iadd(wide_left, wide_right);
                            let wide = self.builder.ins().iadd(wide, wide_carry);
                            let result = self.builder.ins().ireduce(types::I32, wide);
                            let high = self.builder.ins().ushr_imm_s(wide, 32);
                            carry = Some(self.builder.ins().ireduce(types::I32, high));
                            let different_signs = self.builder.ins().bxor(left, right);
                            let same_signs = self.builder.ins().bnot(different_signs);
                            let changed_sign = self.builder.ins().bxor(left, result);
                            let overflow_bit = self.builder.ins().band(same_signs, changed_sign);
                            overflow = Some(self.builder.ins().ushr_imm_s(overflow_bit, 31));
                            result
                        } else {
                            let sum = self.builder.ins().iadd(left, right);
                            self.builder.ins().iadd(sum, carry_in)
                        }
                    }
                    AluOp::And => self.builder.ins().band(left, right),
                    AluOp::Xor => self.builder.ins().bxor(left, right),
                    AluOp::Or => self.builder.ins().bor(left, right),
                    AluOp::Move => right,
                    AluOp::BitClear => {
                        let inverse = self.builder.ins().bnot(right);
                        self.builder.ins().band(left, inverse)
                    }
                    AluOp::Not => self.builder.ins().bnot(right),
                    AluOp::Multiply => {
                        carry = None;
                        self.builder.ins().imul(left, right)
                    }
                    AluOp::CountLeadingZeros => self.builder.ins().clz(right),
                };
                if *set_flags {
                    self.set_flags(result, carry, overflow);
                }
                if let Some(destination) = destination {
                    if *destination == Reg::PC {
                        return Some(self.pc_write(result, false));
                    }
                    self.store_register(destination, result);
                }
            }
            Operation::MultiplyAccumulate {
                destination,
                left,
                right,
                accumulate,
                set_flags,
            } => {
                let left = self.register(left);
                let right = self.register(right);
                let accumulate = self.register(accumulate);
                let product = self.builder.ins().imul(left, right);
                let result = self.builder.ins().iadd(product, accumulate);
                self.store_register(destination, result);
                if *set_flags {
                    self.set_flags(result, None, None);
                }
            }
            Operation::MultiplyLong {
                low,
                high,
                left,
                right,
                signed,
                accumulate,
                set_flags,
            } => {
                let left = self.register(left);
                let right = self.register(right);
                let (left, right) = if *signed {
                    (
                        self.builder.ins().sextend(types::I64, left),
                        self.builder.ins().sextend(types::I64, right),
                    )
                } else {
                    (
                        self.builder.ins().uextend(types::I64, left),
                        self.builder.ins().uextend(types::I64, right),
                    )
                };
                let mut result = self.builder.ins().imul(left, right);
                if *accumulate {
                    let low = self.register(low);
                    let high = self.register(high);
                    let low = self.builder.ins().uextend(types::I64, low);
                    let high = self.builder.ins().uextend(types::I64, high);
                    let high = self.builder.ins().ishl_imm_s(high, 32);
                    let accumulator = self.builder.ins().bor(low, high);
                    result = self.builder.ins().iadd(result, accumulator);
                }
                let low_value = self.builder.ins().ireduce(types::I32, result);
                let high_value = self.builder.ins().ushr_imm_s(result, 32);
                let high_value = self.builder.ins().ireduce(types::I32, high_value);
                self.store_register(low, low_value);
                self.store_register(high, high_value);
                if *set_flags {
                    self.set_flags(result, None, None);
                }
            }
            Operation::ReadCpsr { destination } => {
                let cpsr = self.cpsr();
                self.store_register(destination, cpsr);
            }
            Operation::WriteCpsr { value, mask } => {
                let value = self.value(value);
                let value = self.builder.ins().band_imm_s(value, i64::from(*mask));
                let cpsr = self.cpsr();
                let preserved = self.builder.ins().band_imm_s(cpsr, i64::from(!mask));
                let cpsr = self.builder.ins().bor(preserved, value);
                self.store_frame(offset_of!(RunFrame, cpsr), cpsr);
            }
            Operation::Branch { target, link, exchange } => {
                let target = match target {
                    BranchTarget::Address(address) => self.builder.ins().iconst(types::I32, i64::from(address.get())),
                    BranchTarget::Register(register) => self.register(register),
                };
                let next = self.pc_write(target, *exchange);
                if let Some(link) = link {
                    let link = self.builder.ins().iconst(types::I32, i64::from(link.get()));
                    self.store_register(&Reg::LR, link);
                }
                return Some(next);
            }
            Operation::Load { .. }
            | Operation::Store { .. }
            | Operation::MultipleTransfer { .. }
            | Operation::DoubleTransfer { .. }
            | Operation::Swap { .. } => {
                return self.emit_memory(instruction);
            }
            Operation::Nop => {}
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use cranelift_codegen::{
        control::ControlPlane,
        ir::{AbiParam, InstructionData, Opcode, Signature},
        isa, settings,
    };
    use wie_arm_jit_types::{
        RegionKey,
        ir::{BasicBlock, MemoryAddress},
    };

    use super::*;

    #[test]
    fn fixed_backedge_is_native_and_compiles_for_both_host_isas() {
        let ir = RegionIr {
            entry: RegionKey {
                pc: 0x1000,
                thumb: false,
                cpu_mode: 0x1f,
            },
            blocks: vec![BasicBlock {
                instructions: vec![
                    Instruction {
                        pc: MemoryAddress::new(0x1000),
                        size: 4,
                        condition: Condition::Always,
                        operation: Operation::Alu {
                            op: AluOp::Add,
                            destination: Some(Reg::new(0)),
                            left: ir::Value::Register(Reg::new(0)),
                            right: Operand {
                                value: ir::Value::Immediate(1),
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            set_flags: false,
                        },
                    },
                    Instruction {
                        pc: MemoryAddress::new(0x1004),
                        size: 4,
                        condition: Condition::Always,
                        operation: Operation::Branch {
                            target: BranchTarget::Address(MemoryAddress::new(0x1000)),
                            link: None,
                            exchange: false,
                        },
                    },
                ],
            }],
        };
        for triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"] {
            let isa = isa::lookup(triple.parse().unwrap())
                .unwrap()
                .finish(settings::Flags::new(settings::builder()))
                .unwrap();
            let mut context = Context::new();
            context.func.signature = Signature::new(isa.default_call_conv());
            context.func.signature.params.extend([AbiParam::new(isa.pointer_type()); 2]);
            context.func.signature.returns.push(AbiParam::new(types::I32));
            emit_region(&ir, &mut context, isa.as_ref()).unwrap();
            let blocks: Vec<_> = context.func.layout.blocks().collect();
            let mut backedge = false;
            for (index, &block) in blocks.iter().enumerate() {
                for inst in context.func.layout.block_insts(block) {
                    let data = &context.func.dfg.insts[inst];
                    assert!(!matches!(data.opcode(), Opcode::Call | Opcode::CallIndirect));
                    if let InstructionData::Jump { destination, .. } = data {
                        let target = destination.block(&context.func.dfg.value_lists);
                        backedge |= blocks[..index].contains(&target);
                    }
                }
            }
            assert!(backedge, "fixed guest backedge must stay in native CFG");
            let compiled = context.compile(isa.as_ref(), &mut ControlPlane::default()).unwrap();
            assert!(!compiled.code_buffer().is_empty());
        }
    }
}
