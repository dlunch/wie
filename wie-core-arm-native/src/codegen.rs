use alloc::{collections::BTreeMap, format};
use core::mem::{offset_of, size_of, swap};

use cranelift_codegen::{
    Context,
    ir::{Block, InstBuilder, MemFlagsData, SigRef, Type, Value, condcodes::IntCC, types},
    isa::TargetIsa,
};
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Switch, Variable};

use wie_core_arm_aot::{
    CompiledExit, RunFrame,
    ir::{
        self, AluOp, BranchTarget, Condition, Instruction, MemoryOperand, Operand, Operation, Reg, RegionFlow, RegionIr, RegisterUsage, Shift,
        ShiftAmount, Width,
    },
};
use wie_util::{Result, WieError};

mod memory;

pub(crate) fn emit_region(
    ir: &mut RegionIr,
    flow: RegionFlow,
    context: &mut Context,
    builder_context: &mut FunctionBuilderContext,
    isa: &dyn TargetIsa,
) -> Result<()> {
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
    for block in &mut ir.blocks {
        block.eliminate_dead_alu_stores();
    }
    let mut builder = FunctionBuilder::new(&mut context.func, builder_context);
    let entry = builder.create_block();
    builder.append_block_params_for_function_params(entry);
    builder.switch_to_block(entry);
    let frame = builder.block_params(entry)[0];
    let pages = builder.block_params(entry)[1];
    let transfer = builder.block_params(entry)[2];
    let dispatch = builder.create_block();
    let fault = builder.create_block();
    builder.append_block_param(fault, types::I32);
    builder.append_block_param(fault, types::I32);
    let entries: BTreeMap<_, _> = ir
        .blocks
        .iter()
        .flat_map(|block| &block.instructions)
        .map(|instruction| (instruction.pc.get(), builder.create_block()))
        .collect();
    let mut emitter = Emitter {
        registers: [GuestRegister::Frame; 16],
        translated: [None; 16],
        status: None,
        exchange: None,
        builder,
        frame,
        pages,
        transfer,
        transfer_signature: None,
        ptr_type: isa.pointer_type(),
        thumb: ir.entry.thumb,
        pc: ir.entry.pc,
        mode: u32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 },
        entries,
        dispatch,
        fault,
    };
    let mut usage = RegisterUsage::default();
    for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
        usage.observe(&instruction.operation);
    }
    // Every PC is an external entry. Carry mutable values across those joins for
    // loops with at most one outgoing branch; coalesced fragments otherwise pay for the
    // phi arguments on paths that leave before reusing the register.
    let cache_loop = flow.has_backedge && flow.outgoing_branches <= 1;
    for index in 0..15 {
        let reads = usage.reads[index];
        let writes = usage.writes[index];
        if writes == 0 && reads > 1 {
            let base = emitter.load_frame(offset_of!(RunFrame, regs) + index * size_of::<u32>());
            emitter.registers[index] = GuestRegister::Invariant(base);
            if usage.scalar_bases[index] > 1 {
                let page = emitter.page_pointer(base);
                let unmapped = emitter.builder.ins().icmp_imm_s(IntCC::Equal, page, 0);
                let pointer = emitter.offset_pointer(page, base, &Width::Word);
                emitter.translated[index] = Some((unmapped, pointer));
            }
        } else if cache_loop && writes != 0 && reads + writes > 2 {
            // Amortize the entry load and the exit store.
            let value = emitter.load_frame(offset_of!(RunFrame, regs) + index * size_of::<u32>());
            let var = emitter.builder.declare_var(types::I32);
            emitter.builder.def_var(var, value);
            emitter.registers[index] = GuestRegister::Local(var);
        }
    }
    let cpsr = emitter.load_frame(offset_of!(RunFrame, cpsr));
    if cache_loop {
        emitter.status = Some(StatusVariables {
            control: emitter.builder.declare_var(types::I32),
            nzcv: core::array::from_fn(|_| emitter.builder.declare_var(types::I32)),
        });
        emitter.set_cpsr_bits(u32::MAX, cpsr);
    }
    let pc = emitter.load_frame(offset_of!(RunFrame, regs) + 15 * size_of::<u32>());
    let shift = if ir.entry.thumb { 1 } else { 2 };
    let origin = emitter.entries.first_key_value().map_or(0, |(&pc, _)| pc);
    let offset = emitter.builder.ins().iadd_imm_s(pc, -i64::from(origin));
    let index = emitter.builder.ins().ushr_imm_s(offset, shift);
    emitter.boundaries(cpsr);
    let low = emitter.builder.ins().band_imm_s(pc, (1 << shift) - 1);
    let unaligned = emitter.builder.ins().icmp_imm_s(IntCC::NotEqual, low, 0);
    emitter.guard(unaligned);
    // Instruction-sized keys let Cranelift use dense jump tables for sequential code.
    let mut switch = Switch::new();
    let mut previous = None;
    for (&pc, &block) in &emitter.entries {
        let slot = (pc - origin) >> shift;
        if let Some(previous) = previous
            && slot - previous <= 4
        {
            // A few default entries cost less than another range comparison and branch.
            for hole in previous + 1..slot {
                switch.set_entry(u128::from(hole), dispatch);
            }
        }
        switch.set_entry(u128::from(slot), block);
        previous = Some(slot);
    }
    switch.emit(&mut emitter.builder, index, dispatch);
    for (instruction, flags_live) in ir.blocks.iter().flat_map(|block| &block.instructions).zip(flow.live_flag_updates) {
        emitter.pc = instruction.pc.get();
        emitter.builder.switch_to_block(emitter.entries[&emitter.pc]);
        if emitter.pc < 0x1000 {
            emitter.builder.ins().jump(dispatch, &[]);
            continue;
        }
        let fallthrough = emitter.pc.wrapping_add(u32::from(instruction.size));
        let skipped = if instruction.condition != Condition::Always {
            let condition = emitter.condition(&instruction.condition);
            let execute = emitter.builder.create_block();
            let skip = emitter.builder.create_block();
            emitter.builder.ins().brif(condition, execute, &[], skip, &[]);
            emitter.builder.switch_to_block(execute);
            Some(skip)
        } else {
            None
        };
        let next = emitter.operation(instruction, flags_live);
        let fixed = instruction.fixed_successor(emitter.thumb);
        if let Some(skip) = skipped {
            if next.is_some() || fixed != Some(fallthrough) {
                emitter.commit(next, fixed, fallthrough);
            } else {
                emitter.builder.ins().jump(skip, &[]);
            }
            emitter.builder.switch_to_block(skip);
            emitter.commit(None, Some(fallthrough), fallthrough);
        } else {
            emitter.commit(next, fixed, fallthrough);
        }
    }
    if let Some(exchange) = emitter.exchange {
        emitter.builder.switch_to_block(exchange);
        let value = emitter.builder.block_params(exchange)[0];
        let thumb = emitter.builder.ins().band_imm_s(value, 1);
        let bit = emitter.builder.ins().ishl_imm_s(thumb, 5);
        emitter.set_cpsr_bits(0x20, bit);
        // ARM clears both address bits, Thumb preserves bit one.
        let alignment_bit = emitter.builder.ins().ishl_imm_s(thumb, 1);
        let mask = emitter.builder.ins().bor_imm_s(alignment_bit, -4);
        let next = emitter.builder.ins().band(value, mask);
        emitter.store_register(&Reg::PC, next);
        emitter.builder.ins().jump(dispatch, &[]);
    }
    let publish = emitter.builder.create_block();
    emitter.builder.append_block_param(publish, types::I32);
    for (block, exit) in [(dispatch, CompiledExit::Dispatch), (fault, CompiledExit::GuestFault)] {
        emitter.builder.switch_to_block(block);
        if exit == CompiledExit::GuestFault {
            let pc = emitter.builder.block_params(fault)[0];
            let address = emitter.builder.block_params(fault)[1];
            emitter.store_register(&Reg::PC, pc);
            emitter.store_frame(offset_of!(RunFrame, fault_address), address);
        }
        let reason = emitter.builder.ins().iconst(types::I32, exit as i64);
        emitter.builder.ins().jump(publish, &[reason.into()]);
    }
    emitter.builder.switch_to_block(publish);
    for (index, register) in emitter.registers.into_iter().enumerate() {
        if let GuestRegister::Local(var) = register {
            let value = emitter.builder.use_var(var);
            emitter.store_frame(offset_of!(RunFrame, regs) + index * size_of::<u32>(), value);
        }
    }
    if emitter.status.is_some() {
        let cpsr = emitter.cpsr();
        emitter.store_frame(offset_of!(RunFrame, cpsr), cpsr);
    }
    let reason = emitter.builder.block_params(publish)[0];
    emitter.builder.ins().return_(&[reason]);
    emitter.builder.seal_all_blocks();
    emitter.builder.finalize(isa.frontend_config());
    Ok(())
}

#[derive(Clone, Copy)]
enum GuestRegister {
    Frame,
    Invariant(Value),
    Local(Variable),
}

enum NextPc {
    Direct(Value),
    Exchange(Value),
}

struct StatusVariables {
    control: Variable,
    nzcv: [Variable; 4],
}

struct Emitter<'a> {
    builder: FunctionBuilder<'a>,
    registers: [GuestRegister; 16],
    translated: [Option<(Value, Value)>; 16],
    status: Option<StatusVariables>,
    exchange: Option<Block>,
    frame: Value,
    pages: Value,
    transfer: Value,
    transfer_signature: Option<SigRef>,
    ptr_type: Type,
    thumb: bool,
    pc: u32,
    mode: u32,
    entries: BTreeMap<u32, Block>,
    dispatch: Block,
    fault: Block,
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
            let index = usize::from(register.index());
            match self.registers[index] {
                GuestRegister::Invariant(value) => value,
                GuestRegister::Local(var) => self.builder.use_var(var),
                GuestRegister::Frame => self.load_frame(offset_of!(RunFrame, regs) + index * size_of::<u32>()),
            }
        }
    }

    fn store_register(&mut self, register: &Reg, value: Value) {
        let index = usize::from(register.index());
        if let GuestRegister::Local(var) = self.registers[index] {
            self.builder.def_var(var, value);
        } else {
            self.store_frame(offset_of!(RunFrame, regs) + index * size_of::<u32>(), value);
        }
    }

    fn value(&mut self, value: &ir::Value) -> Value {
        match value {
            ir::Value::Register(register) => self.register(register),
            ir::Value::Immediate(value) => self.builder.ins().iconst(types::I32, i64::from(*value)),
        }
    }

    fn cpsr(&mut self) -> Value {
        let Some(status) = &self.status else {
            return self.load_frame(offset_of!(RunFrame, cpsr));
        };
        let mut cpsr = self.builder.use_var(status.control);
        for (index, flag) in status.nzcv.into_iter().enumerate() {
            let value = self.builder.use_var(flag);
            let bits = self.builder.ins().ishl_imm_s(value, 28 + index as i64);
            cpsr = self.builder.ins().bor(cpsr, bits);
        }
        cpsr
    }

    fn flag(&mut self, bit: u8) -> Value {
        if let Some(status) = &self.status {
            return self.builder.use_var(status.nzcv[usize::from(bit - 28)]);
        }
        let cpsr = self.load_frame(offset_of!(RunFrame, cpsr));
        let value = self.builder.ins().ushr_imm_s(cpsr, i64::from(bit));
        self.builder.ins().band_imm_s(value, 1)
    }

    // The caller has already masked the replacement bits.
    fn set_cpsr_bits(&mut self, mask: u32, bits: Value) {
        if let Some(status) = &self.status {
            let control_mask = mask & 0x0fff_ffff;
            if control_mask != 0 {
                let mut control = self.builder.ins().band_imm_s(bits, i64::from(control_mask));
                if control_mask != 0x0fff_ffff {
                    let previous = self.builder.use_var(status.control);
                    let kept = self.builder.ins().band_imm_s(previous, i64::from(!control_mask));
                    control = self.builder.ins().bor(kept, control);
                }
                self.builder.def_var(status.control, control);
            }
            for (index, flag) in status.nzcv.into_iter().enumerate() {
                if mask & (1 << (28 + index)) != 0 {
                    let shifted = self.builder.ins().ushr_imm_s(bits, 28 + index as i64);
                    let bit = self.builder.ins().band_imm_s(shifted, 1);
                    self.builder.def_var(flag, bit);
                }
            }
        } else {
            let previous = self.load_frame(offset_of!(RunFrame, cpsr));
            let kept = self.builder.ins().band_imm_s(previous, i64::from(!mask));
            let cpsr = self.builder.ins().bor(kept, bits);
            self.store_frame(offset_of!(RunFrame, cpsr), cpsr);
        }
    }

    fn guard(&mut self, condition: Value) {
        let next = self.builder.create_block();
        self.builder.ins().brif(condition, self.dispatch, &[], next, &[]);
        self.builder.switch_to_block(next);
    }

    fn boundaries(&mut self, cpsr: Value) {
        let mode = self.builder.ins().band_imm_s(cpsr, 0x0100_003f);
        let changed = self.builder.ins().icmp_imm_s(IntCC::NotEqual, mode, i64::from(self.mode));
        self.guard(changed);
    }

    fn commit(&mut self, next: Option<NextPc>, fixed: Option<u32>, fallthrough: u32) {
        // Internal edges need no published PC.
        if let Some(pc) = fixed {
            let target = self.entries.get(&pc).copied().filter(|_| pc >= 0x1000);
            if target.is_none() {
                let value = self.builder.ins().iconst(types::I32, i64::from(pc));
                self.store_register(&Reg::PC, value);
            }
            self.builder.ins().jump(target.unwrap_or(self.dispatch), &[]);
        } else {
            let next = match next {
                Some(NextPc::Exchange(value)) => {
                    let exit = *self.exchange.get_or_insert_with(|| {
                        let block = self.builder.create_block();
                        self.builder.append_block_param(block, types::I32);
                        block
                    });
                    self.builder.ins().jump(exit, &[value.into()]);
                    return;
                }
                Some(NextPc::Direct(value)) => value,
                None => self.builder.ins().iconst(types::I32, i64::from(fallthrough)),
            };
            self.store_register(&Reg::PC, next);
            self.builder.ins().jump(self.dispatch, &[]);
        }
    }

    fn pc_write(&mut self, value: Value, exchange: bool) -> NextPc {
        if exchange {
            NextPc::Exchange(value)
        } else {
            NextPc::Direct(self.builder.ins().band_imm_s(value, if self.thumb { -2 } else { -4 }))
        }
    }

    fn operand(&mut self, operand: &Operand, set_carry: bool) -> (Value, Option<Value>) {
        if let Some((value, carry)) = operand.constant_value() {
            let value = self.builder.ins().iconst(types::I32, i64::from(value));
            let carry = carry
                .filter(|_| set_carry)
                .map(|carry| self.builder.ins().iconst(types::I32, i64::from(carry)));
            return (value, carry);
        }
        let value = self.value(&operand.value);
        if operand.shift != Shift::Rrx && operand.amount == ShiftAmount::Immediate(0) {
            return (value, None);
        }
        if let ShiftAmount::Immediate(amount) = operand.amount
            && operand.shift != Shift::Rrx
        {
            let amount = i64::from(amount);
            let (result, carry) = match operand.shift {
                Shift::Lsl if amount <= 32 => {
                    let result = if amount == 32 {
                        self.builder.ins().iconst(types::I32, 0)
                    } else {
                        self.builder.ins().ishl_imm_s(value, amount)
                    };
                    let carry = set_carry.then(|| self.builder.ins().ushr_imm_s(value, 32 - amount));
                    (result, carry)
                }
                Shift::Lsr if amount <= 32 => {
                    let result = if amount == 32 {
                        self.builder.ins().iconst(types::I32, 0)
                    } else {
                        self.builder.ins().ushr_imm_s(value, amount)
                    };
                    let carry = set_carry.then(|| self.builder.ins().ushr_imm_s(value, amount - 1));
                    (result, carry)
                }
                Shift::Lsl | Shift::Lsr => {
                    let zero = self.builder.ins().iconst(types::I32, 0);
                    (zero, set_carry.then_some(zero))
                }
                Shift::Asr => (
                    self.builder.ins().sshr_imm_s(value, amount.min(31)),
                    set_carry.then(|| self.builder.ins().ushr_imm_s(value, (amount - 1).min(31))),
                ),
                Shift::Ror => {
                    let result = self.builder.ins().rotr_imm_s(value, amount & 31);
                    let carry = set_carry.then(|| self.builder.ins().ushr_imm_s(result, 31));
                    (result, carry)
                }
                Shift::Rrx => unreachable!(),
            };
            let carry = carry.map(|carry| self.builder.ins().band_imm_s(carry, 1));
            return (result, carry);
        }
        if operand.shift == Shift::Rrx {
            let carry = self.flag(29);
            let high = self.builder.ins().ishl_imm_s(carry, 31);
            let low = self.builder.ins().ushr_imm_s(value, 1);
            let result = self.builder.ins().bor(high, low);
            let carry = set_carry.then(|| self.builder.ins().band_imm_s(value, 1));
            return (result, carry);
        }
        let amount = match &operand.amount {
            ShiftAmount::Immediate(amount) => self.builder.ins().iconst(types::I32, i64::from(*amount)),
            ShiftAmount::Register(register) => {
                let amount = self.register(register);
                self.builder.ins().band_imm_s(amount, 255)
            }
        };
        let (result, shifted_carry) = match operand.shift {
            Shift::Lsl | Shift::Lsr => {
                // Host shifts mask their counts; ARM instead distinguishes 32 and larger counts.
                let zero = self.builder.ins().iconst(types::I32, 0);
                let within_word = self.builder.ins().icmp_imm_s(IntCC::UnsignedLessThan, amount, 32);
                let shifted = if operand.shift == Shift::Lsl {
                    self.builder.ins().ishl(value, amount)
                } else {
                    self.builder.ins().ushr(value, amount)
                };
                let result = self.builder.ins().select(within_word, shifted, zero);
                let shifted_carry = set_carry.then(|| {
                    let carry_amount = if operand.shift == Shift::Lsl {
                        let bits = self.builder.ins().iconst(types::I32, 32);
                        self.builder.ins().isub(bits, amount)
                    } else {
                        self.builder.ins().iadd_imm_s(amount, -1)
                    };
                    let carry_bit = self.builder.ins().ushr(value, carry_amount);
                    let carry_bit = self.builder.ins().band_imm_s(carry_bit, 1);
                    let has_carry = self.builder.ins().icmp_imm_s(IntCC::UnsignedLessThanOrEqual, amount, 32);
                    self.builder.ins().select(has_carry, carry_bit, zero)
                });
                (result, shifted_carry)
            }
            Shift::Asr => {
                let sign_bit = self.builder.ins().iconst(types::I32, 31);
                let large = self.builder.ins().icmp_imm_s(IntCC::UnsignedGreaterThan, amount, 31);
                let result_amount = self.builder.ins().select(large, sign_bit, amount);
                let result = self.builder.ins().sshr(value, result_amount);
                let shifted_carry = set_carry.then(|| {
                    let previous = self.builder.ins().iadd_imm_s(amount, -1);
                    let carry_amount = self.builder.ins().select(large, sign_bit, previous);
                    let shifted_carry = self.builder.ins().ushr(value, carry_amount);
                    self.builder.ins().band_imm_s(shifted_carry, 1)
                });
                (result, shifted_carry)
            }
            Shift::Ror => {
                let result = self.builder.ins().rotr(value, amount);
                let shifted_carry = set_carry.then(|| self.builder.ins().ushr_imm_s(result, 31));
                (result, shifted_carry)
            }
            Shift::Rrx => unreachable!(),
        };
        let carry = shifted_carry.map(|shifted_carry| {
            let carry = self.flag(29);
            let unchanged = self.builder.ins().icmp_imm_s(IntCC::Equal, amount, 0);
            self.builder.ins().select(unchanged, carry, shifted_carry)
        });
        (result, carry)
    }

    fn set_flags(&mut self, live: u8, result: Value, carry: Option<Value>, overflow: Option<Value>) {
        let mut flags = None;
        // In the packed CPSR the 32-bit sign is already in its destination bit.
        let packed_sign = self.status.is_none() && self.builder.func.dfg.value_type(result) == types::I32;
        if packed_sign && live & 8 != 0 {
            flags = Some(self.builder.ins().band_imm_s(result, i64::from(i32::MIN)));
        }
        let negative = (live & 8 != 0 && !packed_sign).then(|| {
            if self.builder.func.dfg.value_type(result) == types::I64 {
                let bit = self.builder.ins().ushr_imm_s(result, 63);
                self.builder.ins().ireduce(types::I32, bit)
            } else {
                self.builder.ins().ushr_imm_s(result, 31)
            }
        });
        let zero = (live & 4 != 0).then(|| {
            let bit = self.builder.ins().icmp_imm_s(IntCC::Equal, result, 0);
            self.builder.ins().uextend(types::I32, bit)
        });
        for (index, value) in [overflow, carry, zero, negative].into_iter().enumerate() {
            if let Some(value) = value {
                if let Some(status) = &self.status {
                    self.builder.def_var(status.nzcv[index], value);
                } else {
                    let value = self.builder.ins().ishl_imm_s(value, 28 + index as i64);
                    flags = Some(match flags {
                        Some(previous) => self.builder.ins().bor(previous, value),
                        None => value,
                    });
                }
            }
        }
        if let Some(flags) = flags {
            self.set_cpsr_bits(u32::from(live) << 28, flags);
        }
    }

    fn condition(&mut self, condition: &Condition) -> Value {
        let result = match condition {
            Condition::Eq | Condition::Ne | Condition::Cs | Condition::Cc | Condition::Mi | Condition::Pl | Condition::Vs | Condition::Vc => {
                let bit = match condition {
                    Condition::Eq | Condition::Ne => 30,
                    Condition::Cs | Condition::Cc => 29,
                    Condition::Vs | Condition::Vc => 28,
                    _ => 31,
                };
                if self.status.is_some() {
                    self.flag(bit)
                } else {
                    let cpsr = self.load_frame(offset_of!(RunFrame, cpsr));
                    self.builder.ins().band_imm_s(cpsr, 1 << bit)
                }
            }
            Condition::Hi | Condition::Ls => {
                let cpsr = self.cpsr();
                let flags = self.builder.ins().band_imm_s(cpsr, 0x6000_0000);
                self.builder.ins().icmp_imm_s(IntCC::Equal, flags, 0x2000_0000)
            }
            Condition::Ge | Condition::Lt | Condition::Gt | Condition::Le => {
                let cpsr = self.cpsr();
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

    fn operation(&mut self, instruction: &Instruction, flags_live: u8) -> Option<NextPc> {
        match &instruction.operation {
            Operation::Alu {
                op,
                destination,
                left,
                right,
                set_flags,
            } => {
                let set_flags = *set_flags && flags_live != 0;
                if destination.is_none() && !set_flags {
                    return None;
                }
                if (!set_flags || matches!(op, AluOp::Move | AluOp::Not))
                    && let Some((right, carry)) = right.constant_value()
                    && let Some(value) = op.constant_result(left, right)
                {
                    if set_flags {
                        let mask = u32::from(flags_live) << 28;
                        let flags = (value & 0x8000_0000) | (u32::from(value == 0) << 30) | (carry.unwrap_or(0) << 29);
                        let bits = self.builder.ins().iconst(types::I32, i64::from(flags & mask));
                        self.set_cpsr_bits(mask, bits);
                    }
                    if let Some(destination) = destination {
                        let value = self.builder.ins().iconst(types::I32, i64::from(value));
                        if *destination == Reg::PC {
                            return Some(self.pc_write(value, false));
                        }
                        self.store_register(destination, value);
                    }
                    return None;
                }
                let mut left = self.value(left);
                let set_carry = flags_live & 2 != 0 && matches!(op, AluOp::And | AluOp::Xor | AluOp::Or | AluOp::Move | AluOp::BitClear | AluOp::Not);
                let (mut right, shifted_carry) = self.operand(right, set_carry);
                let mut carry = shifted_carry;
                let mut overflow = None;
                let result = match op {
                    AluOp::Add | AluOp::Sub | AluOp::ReverseSub => {
                        if *op == AluOp::ReverseSub {
                            swap(&mut left, &mut right);
                        }
                        let subtract = *op != AluOp::Add;
                        let result = if flags_live & 1 != 0 {
                            let (result, bit) = if subtract {
                                self.builder.ins().ssub_overflow(left, right)
                            } else {
                                self.builder.ins().sadd_overflow(left, right)
                            };
                            overflow = Some(self.builder.ins().uextend(types::I32, bit));
                            result
                        } else if subtract {
                            self.builder.ins().isub(left, right)
                        } else {
                            self.builder.ins().iadd(left, right)
                        };
                        if flags_live & 2 != 0 {
                            let bit = if subtract {
                                self.builder.ins().icmp(IntCC::UnsignedGreaterThanOrEqual, left, right)
                            } else {
                                self.builder.ins().icmp(IntCC::UnsignedLessThan, result, left)
                            };
                            carry = Some(self.builder.ins().uextend(types::I32, bit));
                        }
                        result
                    }
                    AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry => {
                        if *op == AluOp::ReverseSubCarry {
                            swap(&mut left, &mut right);
                        }
                        let subtract = *op != AluOp::AddCarry;
                        if subtract {
                            right = self.builder.ins().bnot(right);
                        }
                        let carry_in = self.flag(29);
                        let result = if flags_live & 2 != 0 {
                            let (sum, first_carry) = self.builder.ins().uadd_overflow(left, right);
                            let (result, last_carry) = self.builder.ins().uadd_overflow(sum, carry_in);
                            let carry_out = self.builder.ins().bor(first_carry, last_carry);
                            carry = Some(self.builder.ins().uextend(types::I32, carry_out));
                            result
                        } else {
                            let sum = self.builder.ins().iadd(left, right);
                            self.builder.ins().iadd(sum, carry_in)
                        };
                        if flags_live & 1 != 0 {
                            let different_signs = self.builder.ins().bxor(left, right);
                            let same_signs = self.builder.ins().bnot(different_signs);
                            let changed_sign = self.builder.ins().bxor(left, result);
                            let overflow_bit = self.builder.ins().band(same_signs, changed_sign);
                            overflow = Some(self.builder.ins().ushr_imm_s(overflow_bit, 31));
                        }
                        result
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
                if set_flags {
                    self.set_flags(flags_live, result, carry, overflow);
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
                if *set_flags && flags_live != 0 {
                    self.set_flags(flags_live, result, None, None);
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
                if *set_flags && flags_live != 0 {
                    self.set_flags(flags_live, result, None, None);
                }
            }
            Operation::ReadCpsr { destination } => {
                let cpsr = self.cpsr();
                self.store_register(destination, cpsr);
            }
            Operation::WriteCpsr { value, mask } => {
                let value = self.value(value);
                let value = self.builder.ins().band_imm_s(value, i64::from(*mask));
                self.set_cpsr_bits(*mask, value);
            }
            Operation::Branch { target, link, exchange } => {
                let next = match target {
                    BranchTarget::Address(address) => {
                        let thumb = if *exchange { address.get() & 1 != 0 } else { self.thumb };
                        if thumb != self.thumb {
                            let bit = self.builder.ins().iconst(types::I32, if thumb { 0x20 } else { 0 });
                            self.set_cpsr_bits(0x20, bit);
                        }
                        NextPc::Direct(
                            self.builder
                                .ins()
                                .iconst(types::I32, i64::from(address.get() & if thumb { !1 } else { !3 })),
                        )
                    }
                    BranchTarget::Register(register) => {
                        let target = self.register(register);
                        self.pc_write(target, *exchange)
                    }
                };
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
    use alloc::{vec, vec::Vec};

    use cranelift_codegen::{
        control::ControlPlane,
        ir::{AbiParam, Opcode, Signature},
        isa, settings,
    };
    use wie_core_arm_aot::{
        RegionKey,
        ir::{BasicBlock, MemoryAddress},
    };

    use super::*;

    #[test]
    fn dead_flags_and_fixed_backedges_compile_for_both_host_isas() {
        let mut ir = RegionIr {
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
                            set_flags: true,
                        },
                    },
                    Instruction {
                        pc: MemoryAddress::new(0x1004),
                        size: 4,
                        condition: Condition::Always,
                        operation: Operation::Store {
                            value: ir::Value::Register(Reg::new(0)),
                            address: MemoryOperand {
                                base: ir::Value::Register(Reg::new(2)),
                                offset: Operand {
                                    value: ir::Value::Immediate(0),
                                    shift: Shift::Lsl,
                                    amount: ShiftAmount::Immediate(0),
                                },
                                subtract: false,
                                pre_index: true,
                                write_back: None,
                            },
                            width: Width::Word,
                        },
                    },
                    Instruction {
                        pc: MemoryAddress::new(0x1008),
                        size: 4,
                        condition: Condition::Always,
                        operation: Operation::Alu {
                            op: AluOp::AddCarry,
                            destination: Some(Reg::new(1)),
                            left: ir::Value::Register(Reg::new(1)),
                            right: Operand {
                                value: ir::Value::Immediate(0),
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            set_flags: false,
                        },
                    },
                    Instruction {
                        pc: MemoryAddress::new(0x100c),
                        size: 4,
                        condition: Condition::Always,
                        operation: Operation::Alu {
                            op: AluOp::Add,
                            destination: Some(Reg::new(1)),
                            left: ir::Value::Register(Reg::new(1)),
                            right: Operand {
                                value: ir::Value::Immediate(1),
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            set_flags: true,
                        },
                    },
                    Instruction {
                        pc: MemoryAddress::new(0x1010),
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
        ir.blocks.extend(
            [
                (AluOp::Add, Shift::Lsl),
                (AluOp::Sub, Shift::Lsl),
                (AluOp::Move, Shift::Lsl),
                (AluOp::Move, Shift::Lsr),
                (AluOp::Move, Shift::Asr),
                (AluOp::Move, Shift::Ror),
                (AluOp::Move, Shift::Rrx),
            ]
            .into_iter()
            .enumerate()
            .map(|(index, (op, shift))| BasicBlock {
                instructions: vec![Instruction {
                    pc: MemoryAddress::new(0x1100 + index as u32 * 8),
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Alu {
                        op,
                        destination: Some(Reg::new(0)),
                        left: ir::Value::Register(Reg::new(0)),
                        right: Operand {
                            value: ir::Value::Register(Reg::new(1)),
                            shift,
                            amount: ShiftAmount::Register(Reg::new(2)),
                        },
                        set_flags: true,
                    },
                }],
            }),
        );
        for triple in ["x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"] {
            let isa = isa::lookup(triple.parse().unwrap())
                .unwrap()
                .finish(settings::Flags::new(settings::builder()))
                .unwrap();
            let mut context = Context::new();
            context.func.signature = Signature::new(isa.default_call_conv());
            context.func.signature.params.extend([AbiParam::new(isa.pointer_type()); 3]);
            context.func.signature.returns.push(AbiParam::new(types::I32));
            let flow = ir.control_flow();
            assert_eq!(flow.live_flag_updates[..5], [2, 0, 0, 0, 0]);
            emit_region(&mut ir, flow, &mut context, &mut FunctionBuilderContext::new(), isa.as_ref()).unwrap();
            let blocks: Vec<_> = context.func.layout.blocks().collect();
            let mut backedge = false;
            let mut calls = 0;
            let mut arithmetic_flags = 0;
            for (index, &block) in blocks.iter().enumerate() {
                for inst in context.func.layout.block_insts(block) {
                    let data = &context.func.dfg.insts[inst];
                    calls += usize::from(matches!(data.opcode(), Opcode::Call | Opcode::CallIndirect));
                    arithmetic_flags += usize::from(matches!(data.opcode(), Opcode::SaddOverflow | Opcode::SsubOverflow));
                    for destination in data.branch_destination(&context.func.dfg.jump_tables, &context.func.dfg.exception_tables) {
                        let target = destination.block(&context.func.dfg.value_lists);
                        backedge |= blocks[..index].contains(&target);
                    }
                }
            }
            assert!(backedge, "fixed guest backedge must stay in native CFG");
            assert_eq!(calls, 0, "persistent native code must not call process-specific flag helpers");
            assert_eq!(arithmetic_flags, 2, "only the two externally visible arithmetic results need overflow");
            let compiled = context.compile(isa.as_ref(), &mut ControlPlane::default()).unwrap();
            assert!(!compiled.code_buffer().is_empty());
            assert!(
                compiled.buffer.relocs().is_empty(),
                "persistent functions must be independently relocatable"
            );
        }
    }
}
