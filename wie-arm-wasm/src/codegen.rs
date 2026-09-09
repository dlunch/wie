use alloc::{collections::BTreeSet, format, string::String, vec, vec::Vec};

use wasm_encoder::{
    BlockType, CodeSection, EntityType, ExportKind, ExportSection, Function, FunctionSection, ImportSection, InstructionSink, MemArg, MemoryType,
    Module, TypeSection, ValType,
};
use wie_arm_jit::{
    Address, AluOp, CompileRequest, CompiledExit, Condition, Instruction, ManifestRegion, Operand, Operation, RegionIr, Shift, ShiftAmount, Value,
    Width,
};

const LEFT: u32 = 2;
const RIGHT: u32 = 3;
const RESULT: u32 = 4;
const PC: u32 = 5;
const CPSR: u32 = 6;
const NEXT_PC: u32 = 7;
const CARRY: u32 = 8;
const AMOUNT: u32 = 9;
const ACCESS_STATUS: u32 = 10;
const WIDE: u32 = 11;

pub struct WasmArtifact {
    pub bytes: Vec<u8>,
    pub manifest: Vec<ManifestRegion>,
}

pub fn compile(request: &CompileRequest) -> Result<WasmArtifact, String> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32; 4], [ValType::I32]);
    types.ty().function([ValType::I32; 4], []);
    types.ty().function([ValType::I32; 2], [ValType::I32]);
    types.ty().function([ValType::I32; 3], [ValType::I32]);
    module.section(&types);
    let mut imports = ImportSection::new();
    imports.import(
        "wie",
        "memory",
        MemoryType {
            minimum: 1,
            maximum: None,
            memory64: false,
            shared: false,
            page_size_log2: None,
        },
    );
    imports.import("wie", "load", EntityType::Function(0));
    imports.import("wie", "store", EntityType::Function(0));
    imports.import("wie", "sample_prepare", EntityType::Function(1));
    imports.import("wie", "word_range", EntityType::Function(3));
    module.section(&imports);
    let mut functions = FunctionSection::new();
    let mut exports = ExportSection::new();
    let mut code = CodeSection::new();
    let mut manifest = Vec::new();
    for (index, region) in request.regions.iter().enumerate() {
        let ir = &region.ir;
        let mut pcs = BTreeSet::new();
        let mut occupied = BTreeSet::new();
        if !matches!(ir.entry.cpu_mode, 0x10 | 0x1f) || ir.blocks.len() > 32 {
            return Err(String::from("unsupported region mode or block count"));
        }
        for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
            let size = if ir.entry.thumb
                && !matches!(
                    instruction.operation,
                    Operation::Branch {
                        target: Value::Immediate(_),
                        link: Some(_),
                        ..
                    }
                ) {
                2
            } else {
                4
            };
            if instruction.size != size
                || instruction.pc % if ir.entry.thumb { 2 } else { 4 } != 0
                || !pcs.insert(instruction.pc)
                || (0..instruction.size).any(|offset| !occupied.insert(instruction.pc.wrapping_add(u32::from(offset))))
            {
                return Err(String::from("invalid instruction boundary"));
            }
            if !supported(&instruction.operation) {
                return Err(String::from("invalid instruction operands"));
            }
        }
        if pcs.len() > 256 || !pcs.contains(&ir.entry.pc) || pcs.first().zip(pcs.last()).is_some_and(|(first, last)| last - first >= 16 * 1024) {
            return Err(String::from("invalid region entry or instruction count"));
        }
        functions.function(2);
        let export = format!("region_{index}");
        exports.export(&export, ExportKind::Func, 4 + index as u32);
        code.function(&compile_region(ir));
        manifest.push(ManifestRegion {
            entry: ir.entry,
            source: region.source.clone(),
            export,
            expected_old: region.expected_old,
        });
    }
    module.section(&functions);
    module.section(&exports);
    module.section(&code);
    let bytes = module.finish();
    if bytes.len() > 512 * 1024 {
        return Err(String::from("encoded module exceeds output limit"));
    }
    Ok(WasmArtifact { bytes, manifest })
}

const fn field(offset: u64) -> MemArg {
    MemArg {
        offset,
        align: 2,
        memory_index: 0,
    }
}

fn compile_region(ir: &RegionIr) -> Function {
    let mut instructions: Vec<_> = ir.blocks.iter().flat_map(|block| &block.instructions).collect();
    instructions.sort_unstable_by_key(|instruction| instruction.pc);
    let first = instructions[0].pc;
    let shift = if ir.entry.thumb { 1 } else { 2 };
    let count = instructions.len() as u32;
    let mut targets = vec![count; ((instructions[instructions.len() - 1].pc - first) >> shift) as usize + 1];
    for (index, instruction) in instructions.iter().enumerate() {
        targets[((instruction.pc - first) >> shift) as usize] = index as u32;
    }
    let mut function = Function::new([(9, ValType::I32), (1, ValType::I64)]);
    let mut s = function.instructions();
    s.loop_(BlockType::Empty);
    s.local_get(0).i32_load(field(60)).local_set(PC);
    s.local_get(PC).i32_const(0x1000).i32_lt_u().if_(BlockType::Empty);
    s.local_get(0).local_get(PC).i32_store(field(84));
    s.i32_const(CompiledExit::GuestFault as i32).return_().end();
    s.local_get(PC).local_get(0).i32_load(field(68)).i32_eq().if_(BlockType::Empty);
    s.i32_const(CompiledExit::End as i32).return_().end();
    for (offset, exit) in [(72, CompiledExit::Budget), (76, CompiledExit::Sample)] {
        s.local_get(0).i32_load(field(offset)).i32_eqz().if_(BlockType::Empty);
        s.i32_const(exit as i32).return_().end();
    }
    s.local_get(0).i32_load(field(64)).local_set(CPSR);
    s.local_get(CPSR).i32_const(0x0100_003f).i32_and();
    s.i32_const(i32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 })
        .i32_ne()
        .if_(BlockType::Empty);
    s.i32_const(CompiledExit::Dispatch as i32).return_().end();

    s.local_get(PC).i32_const((1 << shift) - 1).i32_and().if_(BlockType::Empty);
    s.i32_const(CompiledExit::Dispatch as i32).return_().end();
    // The outer label handles holes and addresses outside the bounded snapshot.
    for _ in 0..=count {
        s.block(BlockType::Empty);
    }
    s.local_get(PC).i32_const(first as i32).i32_sub().i32_const(shift).i32_shr_u();
    s.br_table(targets, count);
    for (index, instruction) in instructions.into_iter().enumerate() {
        s.end();
        s.local_get(0).i32_load(field(76)).i32_const(1).i32_eq().if_(BlockType::Empty);
        s.local_get(1)
            .local_get(PC)
            .local_get(CPSR)
            .local_get(0)
            .i32_load(field(28))
            .call(2)
            .end();
        s.i32_const(instruction.pc.wrapping_add(u32::from(instruction.size)) as i32)
            .local_set(NEXT_PC);
        condition(&mut s, instruction.condition);
        s.if_(BlockType::Empty);
        operation(&mut s, instruction, ir.entry.thumb);
        s.end();
        s.local_get(0).local_get(NEXT_PC).i32_store(field(60));
        for (offset, increment) in [(72, -1), (76, -1), (80, 1)] {
            s.local_get(0)
                .local_get(0)
                .i32_load(field(offset))
                .i32_const(increment)
                .i32_add()
                .i32_store(field(offset));
        }
        s.br(count - index as u32);
    }
    s.end().i32_const(CompiledExit::Dispatch as i32).return_().end();
    s.unreachable().end();
    function
}

fn supported(operation: &Operation) -> bool {
    let valid_value = |value| !matches!(value, Value::Register(16..));
    let valid_operand = |operand: Operand| valid_value(operand.value) && !matches!(operand.amount, ShiftAmount::Register(16..));
    let valid_address =
        |address: Address| address.write_back.is_none_or(|reg| reg < 15) && valid_value(address.base) && valid_operand(address.offset);
    match operation {
        Operation::Alu {
            op,
            destination,
            left,
            right,
            set_flags,
        } => {
            destination.is_none_or(|reg| reg < 15 || (reg == 15 && !set_flags && !matches!(op, AluOp::Multiply | AluOp::CountLeadingZeros)))
                && valid_value(*left)
                && valid_operand(*right)
                && (*op != AluOp::CountLeadingZeros || !set_flags)
        }
        Operation::Branch { target, .. } => valid_value(*target),
        Operation::Load {
            destination,
            address,
            width,
            signed,
        } => (*destination < 15 || (*destination == 15 && *width == Width::Word && !signed)) && valid_address(*address),
        Operation::Store { value, address, width } => {
            valid_value(*value) && (*value != Value::Register(15) || *width == Width::Word) && valid_address(*address)
        }
        Operation::MultiplyAccumulate {
            destination,
            left,
            right,
            accumulate,
            ..
        } => [destination, left, right, accumulate].into_iter().all(|reg| *reg < 15),
        Operation::MultiplyLong { low, high, left, right, .. } => low != high && [low, high, left, right].into_iter().all(|reg| *reg < 15),
        Operation::ReadStatus { destination } => *destination < 15,
        Operation::WriteStatus { value, mask } => valid_value(*value) && mask & 0x0fff_ffff == 0,
        Operation::MultipleTransfer {
            base,
            registers,
            write_back,
            load,
            ..
        } => {
            *base < 15
                && *registers != 0
                && (!write_back || registers & (1 << base) == 0 || (!load && registers.trailing_zeros() == u32::from(*base)))
        }
        Operation::DoubleTransfer { register, address, load } => {
            *register <= 12
                && register % 2 == 0
                && valid_address(*address)
                && (!load || address.write_back.is_none_or(|base| base != *register && base != register + 1))
        }
        Operation::Swap {
            destination,
            address,
            value,
            width,
        } => [destination, address, value].into_iter().all(|reg| *reg < 15) && matches!(width, Width::Byte | Width::Word),
        Operation::Nop => true,
    }
}

fn value(s: &mut InstructionSink<'_>, value: Value, pc: u32, thumb: bool) {
    match value {
        Value::Immediate(value) => {
            s.i32_const(value as i32);
        }
        Value::Register(15) => {
            s.i32_const(pc.wrapping_add(if thumb { 4 } else { 8 }) as i32);
        }
        Value::Register(reg) => {
            s.local_get(0).i32_load(field(u64::from(reg) * 4));
        }
    }
}

fn constant_operand(operand: Operand) -> Option<(u32, Option<u32>)> {
    let (Value::Immediate(value), ShiftAmount::Immediate(amount)) = (operand.value, operand.amount) else {
        return None;
    };
    let (result, carry) = match (operand.shift, u32::from(amount)) {
        (Shift::Rrx, _) => return None,
        (_, 0) => return Some((value, None)),
        (Shift::Lsl, amount) => (value.checked_shl(amount).unwrap_or(0), value.checked_shl(amount - 1).unwrap_or(0) >> 31),
        (Shift::Lsr, amount) => (value.checked_shr(amount).unwrap_or(0), value.checked_shr(amount - 1).unwrap_or(0) & 1),
        (Shift::Asr, amount) => (
            ((value as i32) >> amount.min(31)) as u32,
            (((value as i32) >> (amount - 1).min(31)) as u32) & 1,
        ),
        (Shift::Ror, amount) => {
            let result = value.rotate_right(amount);
            (result, result >> 31)
        }
    };
    Some((result, Some(carry)))
}

fn operand(s: &mut InstructionSink<'_>, operand: Operand, pc: u32, thumb: bool) {
    if let Some((result, carry)) = constant_operand(operand) {
        s.i32_const(result as i32).local_set(RIGHT);
        if let Some(carry) = carry {
            s.i32_const(carry as i32);
        } else {
            flag(s, 29);
        }
        s.local_set(CARRY);
        return;
    }
    value(s, operand.value, pc, thumb);
    s.local_set(RIGHT);
    flag(s, 29);
    s.local_set(CARRY);
    if operand.shift == Shift::Rrx {
        s.local_get(RIGHT)
            .i32_const(1)
            .i32_shr_u()
            .local_get(CARRY)
            .i32_const(31)
            .i32_shl()
            .i32_or();
        s.local_get(RIGHT).i32_const(1).i32_and().local_set(CARRY);
        s.local_set(RIGHT);
        return;
    }
    match operand.amount {
        ShiftAmount::Immediate(0) => return,
        ShiftAmount::Immediate(amount) => {
            s.i32_const(i32::from(amount));
        }
        ShiftAmount::Register(reg) => {
            value(s, Value::Register(reg), pc, thumb);
            s.i32_const(255).i32_and();
        }
    }
    s.local_tee(AMOUNT).if_(BlockType::Empty);
    match operand.shift {
        Shift::Lsl | Shift::Lsr => {
            // Wasm masks shift counts; ARM distinguishes zero, exactly 32, and larger counts.
            s.local_get(AMOUNT).i32_const(32).i32_le_u().if_(BlockType::Empty);
            s.local_get(RIGHT);
            if operand.shift == Shift::Lsl {
                s.i32_const(32).local_get(AMOUNT).i32_sub();
            } else {
                s.local_get(AMOUNT).i32_const(1).i32_sub();
            }
            s.i32_shr_u().i32_const(1).i32_and().local_set(CARRY);
            s.local_get(RIGHT).local_get(AMOUNT);
            if operand.shift == Shift::Lsl {
                s.i32_shl();
            } else {
                s.i32_shr_u();
            }
            s.i32_const(0).local_get(AMOUNT).i32_const(32).i32_lt_u().select().local_set(RIGHT);
            s.else_().i32_const(0).local_set(RIGHT).i32_const(0).local_set(CARRY).end();
        }
        Shift::Asr => {
            s.local_get(AMOUNT)
                .i32_const(32)
                .local_get(AMOUNT)
                .i32_const(32)
                .i32_lt_u()
                .select()
                .local_set(AMOUNT);
            s.local_get(RIGHT)
                .local_get(AMOUNT)
                .i32_const(1)
                .i32_sub()
                .i32_shr_u()
                .i32_const(1)
                .i32_and()
                .local_set(CARRY);
            s.local_get(RIGHT)
                .i64_extend_i32_s()
                .local_get(AMOUNT)
                .i64_extend_i32_u()
                .i64_shr_s()
                .i32_wrap_i64()
                .local_set(RIGHT);
        }
        Shift::Ror => {
            s.local_get(RIGHT)
                .local_get(AMOUNT)
                .i32_rotr()
                .local_tee(RIGHT)
                .i32_const(31)
                .i32_shr_u()
                .local_set(CARRY);
        }
        Shift::Rrx => {}
    }
    s.end();
}

fn flag(s: &mut InstructionSink<'_>, bit: i32) {
    s.local_get(CPSR).i32_const(bit).i32_shr_u().i32_const(1).i32_and();
}

fn condition(s: &mut InstructionSink<'_>, condition: Condition) {
    match condition {
        Condition::Eq | Condition::Ne => {
            flag(s, 30);
        }
        Condition::Cs | Condition::Cc => {
            flag(s, 29);
        }
        Condition::Mi | Condition::Pl => {
            flag(s, 31);
        }
        Condition::Vs | Condition::Vc => {
            flag(s, 28);
        }
        Condition::Hi | Condition::Ls => {
            flag(s, 29);
            flag(s, 30);
            s.i32_eqz().i32_and();
        }
        Condition::Ge | Condition::Lt => {
            flag(s, 31);
            flag(s, 28);
            s.i32_eq();
        }
        Condition::Gt | Condition::Le => {
            flag(s, 31);
            flag(s, 28);
            s.i32_eq();
            flag(s, 30);
            s.i32_eqz().i32_and();
        }
        Condition::Always => {
            s.i32_const(1);
        }
    }
    if matches!(
        condition,
        Condition::Ne | Condition::Cc | Condition::Pl | Condition::Vc | Condition::Ls | Condition::Lt | Condition::Le
    ) {
        s.i32_eqz();
    }
}

fn commit_pc(s: &mut InstructionSink<'_>, thumb: bool, exchange: bool) {
    s.local_set(NEXT_PC);
    if exchange {
        s.local_get(0).local_get(CPSR).i32_const(!0x20).i32_and();
        s.local_get(NEXT_PC)
            .i32_const(1)
            .i32_and()
            .i32_const(5)
            .i32_shl()
            .i32_or()
            .i32_store(field(64));
        s.local_get(NEXT_PC).i32_const(1).i32_and().if_(BlockType::Result(ValType::I32));
        s.i32_const(!1).else_().i32_const(!3).end();
    } else {
        s.i32_const(if thumb { !1 } else { !3 });
    }
    s.local_get(NEXT_PC).i32_and().local_set(NEXT_PC);
}

fn multiply_flags(s: &mut InstructionSink<'_>, wide: bool) {
    s.local_get(0).local_get(CPSR).i32_const(0x3fff_ffff).i32_and();
    if wide {
        s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64();
    } else {
        s.local_get(RESULT);
    }
    s.i32_const(i32::MIN).i32_and().i32_or();
    if wide {
        s.local_get(WIDE).i64_eqz();
    } else {
        s.local_get(RESULT).i32_eqz();
    }
    s.i32_const(30).i32_shl().i32_or().i32_store(field(64));
}

// Capture both the effective address and final writeback before any destination changes.
fn memory_address(s: &mut InstructionSink<'_>, address: Address, pc: u32, thumb: bool) {
    value(s, address.base, pc, thumb);
    s.local_set(LEFT);
    operand(s, address.offset, pc, thumb);
    s.local_get(LEFT).local_get(RIGHT);
    if address.subtract {
        s.i32_sub();
    } else {
        s.i32_add();
    }
    s.local_set(RESULT);
    if address.pre_index {
        s.local_get(RESULT).local_set(LEFT);
    }
}

fn access_result(s: &mut InstructionSink<'_>) {
    s.local_set(ACCESS_STATUS);
    for (status, exit) in [(1, CompiledExit::InterpretOne), (2, CompiledExit::GuestFault)] {
        s.local_get(ACCESS_STATUS).i32_const(status).i32_eq().if_(BlockType::Empty);
        s.i32_const(exit as i32).return_().end();
    }
}

fn word_range(s: &mut InstructionSink<'_>, words: u32) {
    s.local_get(1)
        .local_get(LEFT)
        .i32_const(words as i32)
        .call(3)
        .i32_eqz()
        .if_(BlockType::Empty);
    s.i32_const(CompiledExit::InterpretOne as i32).return_().end();
}

fn transfer_words(s: &mut InstructionSink<'_>, registers: u16, load: bool, pc: u32, thumb: bool) {
    word_range(s, registers.count_ones());
    for (index, register) in (0..16).filter(|reg| registers & (1 << reg) != 0).enumerate() {
        s.local_get(1).local_get(LEFT).i32_const(index as i32 * 4).i32_add().i32_const(4);
        if load {
            s.local_get(0).i32_const(88).i32_add().call(0).drop();
            if register == 15 {
                s.local_get(0).i32_load(field(88));
                commit_pc(s, thumb, true);
            } else {
                s.local_get(0).local_get(0).i32_load(field(88)).i32_store(field(u64::from(register) * 4));
            }
        } else {
            if register == 15 {
                s.i32_const(pc.wrapping_add(12) as i32);
            } else {
                value(s, Value::Register(register), pc, thumb);
            }
            // Admission guarantees success for every word; no late interpreter replay is possible.
            s.call(1).drop();
        }
    }
}

fn operation(s: &mut InstructionSink<'_>, instruction: &Instruction, thumb: bool) {
    match instruction.operation {
        Operation::Alu {
            op,
            destination,
            left,
            right,
            set_flags,
        } => {
            if !set_flags && let (Value::Immediate(left), Some((right, _))) = (left, constant_operand(right)) {
                let result = match op {
                    AluOp::Add => Some(left.wrapping_add(right)),
                    AluOp::Sub => Some(left.wrapping_sub(right)),
                    AluOp::ReverseSub => Some(right.wrapping_sub(left)),
                    AluOp::And => Some(left & right),
                    AluOp::Or => Some(left | right),
                    AluOp::Xor => Some(left ^ right),
                    AluOp::Move => Some(right),
                    AluOp::BitClear => Some(left & !right),
                    AluOp::Not => Some(!right),
                    AluOp::Multiply => Some(left.wrapping_mul(right)),
                    AluOp::CountLeadingZeros => Some(right.leading_zeros()),
                    AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry => None,
                };
                if let Some(result) = result {
                    if let Some(destination) = destination {
                        if destination == 15 {
                            s.i32_const(result as i32);
                            commit_pc(s, thumb, false);
                        } else {
                            s.local_get(0).i32_const(result as i32).i32_store(field(u64::from(destination) * 4));
                        }
                    }
                    return;
                }
            }
            value(s, left, instruction.pc, thumb);
            s.local_set(LEFT);
            operand(s, right, instruction.pc, thumb);
            let arithmetic = matches!(
                op,
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry
            );
            match op {
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry => {
                    if matches!(op, AluOp::ReverseSub | AluOp::ReverseSubCarry) {
                        s.local_get(LEFT).local_get(RIGHT).local_set(LEFT).local_set(RIGHT);
                    }
                    let subtract = !matches!(op, AluOp::Add | AluOp::AddCarry);
                    if subtract {
                        s.local_get(RIGHT).i32_const(-1).i32_xor().local_set(RIGHT);
                    }
                    // A + ~B + C implements subtraction with no-borrow carry semantics.
                    s.local_get(LEFT).i64_extend_i32_u().local_get(RIGHT).i64_extend_i32_u().i64_add();
                    if matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) {
                        flag(s, 29);
                        s.i64_extend_i32_u();
                    } else {
                        s.i64_const(i64::from(subtract));
                    }
                    s.i64_add().local_tee(WIDE).i32_wrap_i64();
                }
                AluOp::And => {
                    s.local_get(LEFT).local_get(RIGHT).i32_and();
                }
                AluOp::Or => {
                    s.local_get(LEFT).local_get(RIGHT).i32_or();
                }
                AluOp::Xor => {
                    s.local_get(LEFT).local_get(RIGHT).i32_xor();
                }
                AluOp::Move => {
                    s.local_get(RIGHT);
                }
                AluOp::BitClear => {
                    s.local_get(LEFT).local_get(RIGHT).i32_const(-1).i32_xor().i32_and();
                }
                AluOp::Not => {
                    s.local_get(RIGHT).i32_const(-1).i32_xor();
                }
                AluOp::Multiply => {
                    s.local_get(LEFT).local_get(RIGHT).i32_mul();
                }
                AluOp::CountLeadingZeros => {
                    s.local_get(RIGHT).i32_clz();
                }
            }
            s.local_set(RESULT);
            if let Some(destination) = destination {
                if destination == 15 {
                    s.local_get(RESULT);
                    commit_pc(s, thumb, false);
                } else {
                    s.local_get(0).local_get(RESULT).i32_store(field(u64::from(destination) * 4));
                }
            }
            if set_flags {
                s.local_get(0)
                    .local_get(CPSR)
                    .i32_const(if arithmetic {
                        0x0fff_ffff
                    } else if op == AluOp::Multiply {
                        0x3fff_ffff
                    } else {
                        0x1fff_ffff
                    })
                    .i32_and();
                s.local_get(RESULT).i32_const(i32::MIN).i32_and().i32_or();
                s.local_get(RESULT).i32_eqz().i32_const(30).i32_shl().i32_or();
                if arithmetic {
                    s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64();
                    s.i32_const(29).i32_shl().i32_or();
                    // Overflow uses the normalized addends, including the complemented subtrahend.
                    s.local_get(LEFT).local_get(RIGHT).i32_xor().i32_const(-1).i32_xor();
                    s.local_get(LEFT).local_get(RESULT).i32_xor().i32_and();
                    s.i32_const(31).i32_shr_u().i32_const(28).i32_shl().i32_or();
                } else if op != AluOp::Multiply {
                    s.local_get(CARRY).i32_const(29).i32_shl().i32_or();
                }
                s.i32_store(field(64));
            }
        }
        Operation::Branch { target, link, exchange } => {
            value(s, target, instruction.pc, thumb);
            commit_pc(s, thumb, exchange);
            if let Some(link) = link {
                s.local_get(0).i32_const(link as i32).i32_store(field(56));
            }
        }
        Operation::Load { address, width, .. } | Operation::Store { address, width, .. } => {
            memory_address(s, address, instruction.pc, thumb);
            s.local_get(1).local_get(LEFT);
            s.i32_const(match width {
                Width::Byte => 1,
                Width::Half => 2,
                Width::Word => 4,
            });
            if let Operation::Store { value: source, .. } = instruction.operation {
                if source == Value::Register(15) {
                    s.i32_const(instruction.pc.wrapping_add(12) as i32);
                } else {
                    value(s, source, instruction.pc, thumb);
                }
                s.call(1);
            } else {
                s.local_get(0).i32_const(88).i32_add().call(0);
            }
            access_result(s);
            // Helpers may decline an access without side effects. Commit registers only after success.
            if let Operation::Load { destination, signed, .. } = instruction.operation {
                if destination != 15 {
                    s.local_get(0);
                }
                s.local_get(0).i32_load(field(88));
                if signed {
                    match width {
                        Width::Byte => {
                            s.i32_extend8_s();
                        }
                        Width::Half => {
                            s.i32_extend16_s();
                        }
                        Width::Word => {}
                    }
                }
                if destination == 15 {
                    commit_pc(s, thumb, true);
                } else {
                    s.i32_store(field(u64::from(destination) * 4));
                }
            }
            if let Some(register) = address.write_back {
                s.local_get(0).local_get(RESULT).i32_store(field(u64::from(register) * 4));
            }
        }
        Operation::MultiplyAccumulate {
            destination,
            left,
            right,
            accumulate,
            set_flags,
        } => {
            value(s, Value::Register(left), instruction.pc, thumb);
            value(s, Value::Register(right), instruction.pc, thumb);
            s.i32_mul();
            value(s, Value::Register(accumulate), instruction.pc, thumb);
            s.i32_add().local_set(RESULT);
            s.local_get(0).local_get(RESULT).i32_store(field(u64::from(destination) * 4));
            if set_flags {
                multiply_flags(s, false);
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
            for register in [left, right] {
                value(s, Value::Register(register), instruction.pc, thumb);
                if signed {
                    s.i64_extend_i32_s();
                } else {
                    s.i64_extend_i32_u();
                }
            }
            s.i64_mul();
            if accumulate {
                value(s, Value::Register(low), instruction.pc, thumb);
                s.i64_extend_i32_u();
                value(s, Value::Register(high), instruction.pc, thumb);
                s.i64_extend_i32_u().i64_const(32).i64_shl().i64_or().i64_add();
            }
            s.local_set(WIDE);
            s.local_get(0).local_get(WIDE).i32_wrap_i64().i32_store(field(u64::from(low) * 4));
            s.local_get(0)
                .local_get(WIDE)
                .i64_const(32)
                .i64_shr_u()
                .i32_wrap_i64()
                .i32_store(field(u64::from(high) * 4));
            if set_flags {
                multiply_flags(s, true);
            }
        }
        Operation::ReadStatus { destination } => {
            s.local_get(0).local_get(CPSR).i32_store(field(u64::from(destination) * 4));
        }
        Operation::WriteStatus { value: source, mask } => {
            s.local_get(0).local_get(CPSR).i32_const(!mask as i32).i32_and();
            value(s, source, instruction.pc, thumb);
            s.i32_const(mask as i32).i32_and().i32_or().i32_store(field(64));
        }
        Operation::MultipleTransfer {
            base,
            registers,
            increment,
            before,
            write_back,
            load,
        } => {
            let bytes = registers.count_ones() as i32 * 4;
            value(s, Value::Register(base), instruction.pc, thumb);
            s.local_tee(LEFT)
                .i32_const(if increment { bytes } else { -bytes })
                .i32_add()
                .local_set(RESULT);
            s.local_get(LEFT).i32_const(if increment {
                if before { 4 } else { 0 }
            } else {
                -bytes + if before { 0 } else { 4 }
            });
            s.i32_add().local_set(LEFT);
            transfer_words(s, registers, load, instruction.pc, thumb);
            if write_back {
                s.local_get(0).local_get(RESULT).i32_store(field(u64::from(base) * 4));
            }
        }
        Operation::DoubleTransfer { register, address, load } => {
            memory_address(s, address, instruction.pc, thumb);
            transfer_words(s, 3 << register, load, instruction.pc, thumb);
            if let Some(base) = address.write_back {
                s.local_get(0).local_get(RESULT).i32_store(field(u64::from(base) * 4));
            }
        }
        Operation::Swap {
            destination,
            address,
            value: source,
            width,
        } => {
            value(s, Value::Register(address), instruction.pc, thumb);
            s.local_set(LEFT);
            value(s, Value::Register(source), instruction.pc, thumb);
            s.local_set(RIGHT);
            if width == Width::Word {
                word_range(s, 1);
            }
            let bytes = if width == Width::Word { 4 } else { 1 };
            s.local_get(1)
                .local_get(LEFT)
                .i32_const(bytes)
                .local_get(0)
                .i32_const(88)
                .i32_add()
                .call(0);
            if width == Width::Byte {
                access_result(s);
            } else {
                s.drop();
            }
            s.local_get(1).local_get(LEFT).i32_const(bytes).local_get(RIGHT).call(1).drop();
            s.local_get(0)
                .local_get(0)
                .i32_load(field(88))
                .i32_store(field(u64::from(destination) * 4));
        }
        Operation::Nop => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use wie_arm_jit::{BasicBlock, CodePageStamp, CompileRegion, RegionKey};

    use super::*;

    #[test]
    fn encoded_modules_exceeding_the_output_limit_are_rejected() {
        let mut request = CompileRequest {
            session: 1,
            request: 1,
            regions: (0..8)
                .map(|region| {
                    let pc = 0x100000 + region * 0x1000;
                    CompileRegion {
                        ir: RegionIr {
                            entry: RegionKey {
                                pc,
                                thumb: false,
                                cpu_mode: 0x10,
                            },
                            blocks: vec![BasicBlock {
                                instructions: (0..256)
                                    .map(|index| Instruction {
                                        pc: pc + index * 4,
                                        size: 4,
                                        condition: Condition::Le,
                                        operation: Operation::Alu {
                                            op: AluOp::ReverseSubCarry,
                                            destination: Some(0),
                                            left: Value::Register(0),
                                            right: Operand {
                                                value: Value::Register(1),
                                                shift: Shift::Lsl,
                                                amount: ShiftAmount::Register(2),
                                            },
                                            set_flags: true,
                                        },
                                    })
                                    .collect(),
                            }],
                        },
                        source: vec![CodePageStamp {
                            page: pc & 0xffff0000,
                            version: 1,
                        }],
                        expected_old: None,
                    }
                })
                .collect(),
        };
        assert_eq!(compile(&request).err().as_deref(), Some("encoded module exceeds output limit"));
        request.regions.truncate(6);
        let artifact = compile(&request).unwrap();
        assert_eq!(artifact.manifest.len(), 6);
        assert!((384 * 1024..=512 * 1024).contains(&artifact.bytes.len()));
    }

    #[test]
    fn immediate_shifts_emit_constant_values_and_preserve_dynamic_carry() {
        for (shift, amount, input, result, carry) in [
            (Shift::Ror, 2, 2, 0x8000_0000, Some(1)),
            (Shift::Ror, 2, 4, 1, Some(0)),
            (Shift::Ror, 0, 2, 2, None),
            (Shift::Ror, 32, 0x8000_0001, 0x8000_0001, Some(1)),
            (Shift::Lsl, 0, 2, 2, None),
            (Shift::Lsl, 1, 0x8000_0001, 2, Some(1)),
            (Shift::Lsl, 32, 1, 0, Some(1)),
            (Shift::Lsl, 33, 1, 0, Some(0)),
            (Shift::Lsr, 1, 3, 1, Some(1)),
            (Shift::Lsr, 32, 0x8000_0000, 0, Some(1)),
            (Shift::Lsr, 33, 0x8000_0000, 0, Some(0)),
            (Shift::Asr, 1, 0x8000_0001, 0xc000_0000, Some(1)),
            (Shift::Asr, 32, 0x8000_0000, u32::MAX, Some(1)),
            (Shift::Asr, 255, 0x7fff_ffff, 0, Some(0)),
        ] {
            let mut actual = Function::new([]);
            operand(
                &mut actual.instructions(),
                Operand {
                    value: Value::Immediate(input),
                    shift,
                    amount: ShiftAmount::Immediate(amount),
                },
                0x1000,
                false,
            );
            let mut expected = Function::new([]);
            let mut s = expected.instructions();
            s.i32_const(result as i32).local_set(RIGHT);
            if let Some(carry) = carry {
                s.i32_const(carry);
            } else {
                s.local_get(CPSR).i32_const(29).i32_shr_u().i32_const(1).i32_and();
            }
            s.local_set(CARRY);
            assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{shift:?} {input:#x} by {amount}");
        }
    }

    #[test]
    fn pc_relative_immediate_alu_emits_only_a_constant_store() {
        for (op, left, input, shift, amount, result) in [
            (AluOp::Add, 0x1008, 4, Shift::Lsl, 0, 0x100c_u32),
            (AluOp::Sub, 0x1008, 4, Shift::Lsl, 0, 0x1004),
            (AluOp::ReverseSub, 0x1008, 4, Shift::Lsl, 0, 0xffff_effc),
            (AluOp::Add, 0x1008, 2, Shift::Ror, 2, 0x8000_1008),
            (AluOp::Add, 0xffff_fffc, 8, Shift::Lsl, 0, 4),
            (AluOp::And, 0x1008, 0xff, Shift::Lsl, 0, 8),
            (AluOp::Or, 0x1008, 4, Shift::Lsl, 0, 0x100c),
            (AluOp::Xor, 0x1008, 8, Shift::Lsl, 0, 0x1000),
            (AluOp::BitClear, 0x1008, 8, Shift::Lsl, 0, 0x1000),
            (AluOp::Move, 0, 2, Shift::Ror, 2, 0x8000_0000),
            (AluOp::Not, 0, 0xff, Shift::Lsl, 0, 0xffff_ff00),
            (AluOp::Multiply, 0x8000_0001, 2, Shift::Lsl, 0, 2),
        ] {
            let mut actual = Function::new([]);
            operation(
                &mut actual.instructions(),
                &Instruction {
                    pc: 0x1000,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Alu {
                        op,
                        destination: Some(0),
                        left: Value::Immediate(left),
                        right: Operand {
                            value: Value::Immediate(input),
                            shift,
                            amount: ShiftAmount::Immediate(amount),
                        },
                        set_flags: false,
                    },
                },
                false,
            );
            let mut expected = Function::new([]);
            expected.instructions().local_get(0).i32_const(result as i32).i32_store(field(0));
            assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{op:?} {left:#x}, {input:#x}");
        }
    }
}
