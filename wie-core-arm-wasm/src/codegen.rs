use alloc::{collections::VecDeque, format, string::String, vec, vec::Vec};

use wasm_encoder::{
    BlockType, Encode, EntityType, ExportKind, Function, ImportSection, InstructionSink, MemArg, MemoryType, Module, SectionId, TypeSection, ValType,
};
use wie_arm_jit_types::CompiledExit;
use wie_arm_jit_types::ir::{Address, AluOp, Condition, Instruction, Operand, Operation, RegionIr, Shift, ShiftAmount, Value, Width};

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
const RANGE_FIRST: u32 = 12;
const RANGE_SECOND: u32 = 13;
const RANGE_LENGTH: u32 = 14;
const CALL_TAKEN: u32 = 15;
const EXECUTED: u32 = 16;
const BUDGET: u32 = 17;
const SAMPLE: u32 = 18;
const END: u32 = 19;

#[derive(Default)]
pub(crate) struct ModuleBuilder {
    functions: Vec<u8>,
    exports: Vec<u8>,
    bodies: VecDeque<Vec<u8>>,
    code_size: usize,
}

impl ModuleBuilder {
    pub(crate) fn add_region(&mut self, ir: &RegionIr) -> String {
        let index = self.functions.len() as u32;
        let export = format!("region_{index}");
        2_u32.encode(&mut self.functions);
        export.encode(&mut self.exports);
        ExportKind::Func.encode(&mut self.exports);
        (4 + index).encode(&mut self.exports);
        let body = compile_region(ir).into_raw_body();
        let mut length = Vec::new();
        (body.len() as u32).encode(&mut length);
        self.code_size += length.len() + body.len();
        self.bodies.push_back(length);
        self.bodies.push_back(body);
        export
    }

    pub(crate) fn begin_assembly(mut self, output: &mut Vec<u8>) -> Result<VecDeque<Vec<u8>>, String> {
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32; 4], [ValType::I32]);
        types.ty().function([ValType::I32; 3], []);
        types.ty().function([ValType::I32; 2], [ValType::I32]);
        types.ty().function([ValType::I32; 3], [ValType::I64]);
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
        let prefix = module.finish();
        let count = self.functions.len() as u32;
        let mut count_bytes = Vec::new();
        count.encode(&mut count_bytes);
        let [functions_header, exports_header, code_header] = [
            (SectionId::Function, self.functions.len()),
            (SectionId::Export, self.exports.len()),
            (SectionId::Code, self.code_size),
        ]
        .map(|(id, length)| {
            let mut header = vec![id.into()];
            let size = u32::try_from(length + count_bytes.len()).map_err(|_| String::from("Wasm section exceeds 4 GiB"))?;
            size.encode(&mut header);
            header.extend_from_slice(&count_bytes);
            Ok::<_, String>(header)
        });
        let (functions_header, exports_header, code_header) = (functions_header?, exports_header?, code_header?);
        // Reserve once; subsequent assembly copies at most one chunk and never reallocates the output.
        output.reserve_exact(
            prefix.len()
                + self.functions.len()
                + self.exports.len()
                + self.code_size
                + functions_header.len()
                + exports_header.len()
                + code_header.len(),
        );
        self.bodies.push_front(code_header);
        self.bodies.push_front(self.exports);
        self.bodies.push_front(exports_header);
        self.bodies.push_front(self.functions);
        self.bodies.push_front(functions_header);
        self.bodies.push_front(prefix);
        Ok(self.bodies)
    }
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
    let count = ir.blocks.len() as u32;
    let mut targets = vec![count; ((instructions[instructions.len() - 1].pc - first) >> shift) as usize + 1];
    for (index, block) in ir.blocks.iter().enumerate() {
        for instruction in &block.instructions {
            targets[((instruction.pc - first) >> shift) as usize] = index as u32;
        }
    }
    let mut function = Function::new([(9, ValType::I32), (1, ValType::I64), (8, ValType::I32)]);
    let mut s = function.instructions();
    s.local_get(0).i32_load(field(72)).local_set(BUDGET);
    s.local_get(0).i32_load(field(76)).local_set(SAMPLE);
    s.local_get(0).i32_load(field(68)).local_set(END);
    s.local_get(0).i32_load(field(64)).local_set(CPSR);
    s.block(BlockType::Result(ValType::I32));
    s.loop_(BlockType::Empty);
    boundaries(&mut s, ir, None, 1);
    for _ in 0..=count {
        s.block(BlockType::Empty);
    }
    s.local_get(PC).i32_const(first as i32).i32_sub().i32_const(shift).i32_shr_u();
    s.br_table(targets, count);
    for (index, block) in ir.blocks.iter().enumerate() {
        s.end();
        if block.instructions.len() > 1 {
            let first = block.instructions[0].pc;
            let last = block.instructions.last().unwrap().pc;
            let mut targets = vec![0; ((last - first) >> shift) as usize + 1];
            for (index, instruction) in block.instructions.iter().enumerate() {
                targets[((instruction.pc - first) >> shift) as usize] = index as u32;
                s.block(BlockType::Empty);
            }
            // The outer selector has already rejected holes and unknown PCs.
            s.local_get(PC).i32_const(first as i32).i32_sub().i32_const(shift).i32_shr_u();
            s.br_table(targets, 0);
        }
        for (instruction_index, instruction) in block.instructions.iter().enumerate() {
            if block.instructions.len() > 1 {
                s.end();
            }
            let exit_depth = count - index as u32 + (block.instructions.len() - instruction_index) as u32;
            boundaries(&mut s, ir, Some(instruction.pc), exit_depth);
            s.local_get(SAMPLE)
                .local_get(EXECUTED)
                .i32_sub()
                .i32_const(1)
                .i32_eq()
                .if_(BlockType::Empty);
            s.local_get(1).local_get(PC).local_get(0).i32_load(field(28)).call(2).end();
            s.i32_const(instruction.pc.wrapping_add(u32::from(instruction.size)) as i32)
                .local_set(NEXT_PC);
            let call = matches!(instruction.operation, Operation::Branch { link: Some(_), .. });
            condition(&mut s, instruction.condition);
            if call {
                s.local_tee(CALL_TAKEN);
            }
            s.if_(BlockType::Empty);
            operation(&mut s, instruction, ir.entry.thumb, exit_depth + 1);
            s.end();
            s.local_get(0).local_get(NEXT_PC).i32_store(field(60));
            s.local_get(EXECUTED).i32_const(1).i32_add().local_set(EXECUTED);
            if call {
                s.local_get(CALL_TAKEN).if_(BlockType::Empty);
                boundaries(&mut s, ir, None, exit_depth + 1);
                s.i32_const(CompiledExit::Dispatch as i32).br(exit_depth + 1).end();
            }
        }
        s.br(count - index as u32);
    }
    s.end().i32_const(CompiledExit::Dispatch as i32).br(1).end();
    s.unreachable().end();
    // Every logical exit commits the completed prefix once; the exit reason stays on the stack.
    for (offset, initial) in [(72, BUDGET), (76, SAMPLE)] {
        s.local_get(0).local_get(initial).local_get(EXECUTED).i32_sub().i32_store(field(offset));
    }
    s.local_get(0)
        .local_get(0)
        .i32_load(field(80))
        .local_get(EXECUTED)
        .i32_add()
        .i32_store(field(80));
    s.end();
    function
}

fn boundaries(s: &mut InstructionSink<'_>, ir: &RegionIr, instruction_pc: Option<u32>, exit_depth: u32) {
    if let Some(pc) = instruction_pc {
        s.i32_const(pc as i32).local_set(PC);
    } else {
        s.local_get(0).i32_load(field(60)).local_set(PC);
        s.local_get(PC).i32_const(0x1000).i32_lt_u().if_(BlockType::Empty);
        s.local_get(0).local_get(PC).i32_store(field(84));
        s.i32_const(CompiledExit::GuestFault as i32).br(exit_depth + 1).end();
    }
    s.local_get(PC).local_get(END).i32_eq().if_(BlockType::Empty);
    s.i32_const(CompiledExit::End as i32).br(exit_depth + 1).end();
    for (initial, exit) in [(BUDGET, CompiledExit::Budget), (SAMPLE, CompiledExit::Sample)] {
        s.local_get(EXECUTED).local_get(initial).i32_eq().if_(BlockType::Empty);
        s.i32_const(exit as i32).br(exit_depth + 1).end();
    }
    if instruction_pc.is_none() {
        s.local_get(CPSR).i32_const(0x0100_003f).i32_and();
        s.i32_const(i32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 })
            .i32_ne()
            .if_(BlockType::Empty);
        s.i32_const(CompiledExit::Dispatch as i32).br(exit_depth + 1).end();
        s.local_get(PC)
            .i32_const(if ir.entry.thumb { 1 } else { 3 })
            .i32_and()
            .if_(BlockType::Empty);
        s.i32_const(CompiledExit::Dispatch as i32).br(exit_depth + 1).end();
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
            .local_tee(CPSR)
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
    s.i32_const(30).i32_shl().i32_or().local_tee(CPSR).i32_store(field(64));
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

fn access_result(s: &mut InstructionSink<'_>, exit_depth: u32) {
    s.local_set(ACCESS_STATUS);
    for (status, exit) in [(1, CompiledExit::InterpretOne), (2, CompiledExit::GuestFault)] {
        s.local_get(ACCESS_STATUS).i32_const(status).i32_eq().if_(BlockType::Empty);
        s.i32_const(exit as i32).br(exit_depth + 1).end();
    }
}

fn word_range(s: &mut InstructionSink<'_>, words: u32, exit_depth: u32) {
    s.local_get(1)
        .local_get(LEFT)
        .i32_const(words as i32)
        .call(3)
        .local_tee(WIDE)
        .i64_eqz()
        .if_(BlockType::Empty);
    s.i32_const(CompiledExit::InterpretOne as i32).br(exit_depth + 1).end();
    s.local_get(WIDE).i32_wrap_i64().local_set(RANGE_FIRST);
    s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64().local_set(RANGE_SECOND);
    s.local_get(0).i32_load(field(88)).local_set(RANGE_LENGTH);
}

fn transfer_words(s: &mut InstructionSink<'_>, registers: u16, load: bool, pc: u32, thumb: bool, exit_depth: u32) {
    word_range(s, registers.count_ones(), exit_depth);
    for (index, register) in (0..16).filter(|reg| registers & (1 << reg) != 0).enumerate() {
        if load && register != 15 {
            s.local_get(0);
        }
        s.local_get(RANGE_FIRST);
        if index != 0 {
            let offset = index as i32 * 4;
            // Both addresses are integers; only the selected, admitted span is dereferenced.
            s.i32_const(offset).i32_add();
            s.local_get(RANGE_SECOND).i32_const(offset).i32_add().local_get(RANGE_LENGTH).i32_sub();
            s.i32_const(offset).local_get(RANGE_LENGTH).i32_lt_u().select();
        }
        if load {
            s.i32_load(field(0));
            if register == 15 {
                commit_pc(s, thumb, true);
            } else {
                s.i32_store(field(u64::from(register) * 4));
            }
        } else {
            if register == 15 {
                s.i32_const(pc.wrapping_add(12) as i32);
            } else {
                value(s, Value::Register(register), pc, thumb);
            }
            s.i32_store(field(0));
        }
    }
}

fn operation(s: &mut InstructionSink<'_>, instruction: &Instruction, thumb: bool, exit_depth: u32) {
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
                s.local_tee(CPSR).i32_store(field(64));
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
            access_result(s, exit_depth);
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
            s.i32_const(mask as i32).i32_and().i32_or().local_tee(CPSR).i32_store(field(64));
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
            transfer_words(s, registers, load, instruction.pc, thumb, exit_depth);
            if write_back {
                s.local_get(0).local_get(RESULT).i32_store(field(u64::from(base) * 4));
            }
        }
        Operation::DoubleTransfer { register, address, load } => {
            memory_address(s, address, instruction.pc, thumb);
            transfer_words(s, 3 << register, load, instruction.pc, thumb, exit_depth);
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
                word_range(s, 1, exit_depth);
                s.local_get(RANGE_FIRST).i32_load(field(0)).local_set(RESULT);
                s.local_get(RANGE_FIRST).local_get(RIGHT).i32_store(field(0));
            } else {
                s.local_get(1).local_get(LEFT).i32_const(1).local_get(0).i32_const(88).i32_add().call(0);
                access_result(s, exit_depth);
                s.local_get(1).local_get(LEFT).i32_const(1).local_get(RIGHT).call(1).drop();
                s.local_get(0).i32_load(field(88)).local_set(RESULT);
            }
            s.local_get(0).local_get(RESULT).i32_store(field(u64::from(destination) * 4));
        }
        Operation::Nop => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use wie_arm_jit_types::{CodePageStamp, CompileRegion, CompileRequest, RegionKey, ir::BasicBlock};

    use crate::Compiler;

    use super::*;

    #[test]
    fn large_batches_compile_into_a_single_module() {
        let request: CompileRequest = Box::new((0..16).map(|step| {
            if step % 2 == 0 {
                return None;
            }
            let pc = 0x100000 + (step / 2) * 0x1000;
            Some(CompileRegion {
                ir: RegionIr {
                    entry: RegionKey {
                        pc,
                        thumb: false,
                        cpu_mode: 0x1f,
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
            })
        }));
        let mut compiler = Compiler::new(request);
        assert!(!compiler.step().unwrap());
        assert!(compiler.artifact.manifest.is_empty());
        assert!(!compiler.step().unwrap());
        assert_eq!(compiler.artifact.manifest.len(), 1);
        let mut assembly_steps = 0;
        loop {
            let previous = compiler.artifact.bytes.len();
            let complete = compiler.step().unwrap();
            let copied = compiler.artifact.bytes.len() - previous;
            assert!(copied <= 64 * 1024);
            assembly_steps += usize::from(copied != 0);
            if complete {
                break;
            }
        }
        assert!(assembly_steps > 8);
        let artifact = compiler.finish();
        assert_eq!(artifact.manifest.iter().filter(|region| !region.entry.thumb).count(), 8);
        assert!(artifact.bytes.len() > 512 * 1024);
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
                0,
            );
            let mut expected = Function::new([]);
            expected.instructions().local_get(0).i32_const(result as i32).i32_store(field(0));
            assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{op:?} {left:#x}, {input:#x}");
        }
    }
}
