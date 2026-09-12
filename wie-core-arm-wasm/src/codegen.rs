use alloc::{borrow::Cow, collections::VecDeque, string::String, vec, vec::Vec};

use wasm_encoder::{
    BlockType, ConstExpr, ElementSection, Elements, Encode, EntityType, ExportKind, ExportSection, Function, ImportSection, InstructionSink, MemArg,
    MemoryType, Module, RefType, SectionId, TableSection, TableType, TypeSection, ValType,
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
const EXECUTED: u32 = 15;
const PAGE_TABLE: u32 = 16;
const PAGE_POINTER: u32 = 17;
const SAMPLE_AT: u32 = 18;
const END: u32 = 19;

// Function indices follow the import and helper section order.
const PAGES: u32 = 0;
const SAMPLE: u32 = 1;
const WORD_RANGE: u32 = 2;
const RESOLVE: u32 = 3;
const BOUNDARY: u32 = 4;
const ENTRY: u32 = 5;
const COMMIT: u32 = 6;
const FIRST_REGION: u32 = 7;

pub(crate) struct ModuleBuilder {
    functions: Vec<u8>,
    bodies: VecDeque<Vec<u8>>,
    code_size: usize,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        let mut builder = Self {
            functions: vec![0, 0, 5],
            bodies: VecDeque::new(),
            code_size: 0,
        };
        let mut boundary = Function::new([]);
        let mut s = boundary.instructions();
        for (value, offset, exit) in [(2, 68, CompiledExit::End), (3, 72, CompiledExit::Sample)] {
            s.local_get(value).local_get(0).i32_load(field(offset)).i32_eq().if_(BlockType::Empty);
            s.i32_const(exit as i32).return_().end();
        }
        s.i32_const(-1).end();
        builder.push_body(boundary);
        let mut entry = Function::new([]);
        let mut s = entry.instructions();
        s.local_get(1).i32_const(0x1000).i32_lt_u().if_(BlockType::Empty);
        s.local_get(0).local_get(1).i32_store(field(80));
        s.i32_const(CompiledExit::GuestFault as i32).return_().end();
        for (value, offset, exit) in [(1, 68, CompiledExit::End), (2, 72, CompiledExit::Sample)] {
            s.local_get(value).local_get(0).i32_load(field(offset)).i32_eq().if_(BlockType::Empty);
            s.i32_const(exit as i32).return_().end();
        }
        s.local_get(0).i32_load(field(64)).i32_const(0x0100_003f).i32_and();
        s.local_get(3).i32_ne().if_(BlockType::Empty);
        s.i32_const(CompiledExit::Dispatch as i32).return_().end();
        // T selects two-byte alignment; ARM requires four-byte alignment.
        s.local_get(1)
            .local_get(3)
            .i32_const(4)
            .i32_shr_u()
            .i32_const(2)
            .i32_and()
            .i32_const(3)
            .i32_xor()
            .i32_and();
        s.if_(BlockType::Empty).i32_const(CompiledExit::Dispatch as i32).return_().end();
        s.i32_const(-1).end();
        builder.push_body(entry);
        let mut commit = Function::new([]);
        let mut s = commit.instructions();
        s.local_get(0)
            .local_get(0)
            .i32_load(field(72))
            .local_get(1)
            .i32_sub()
            .i32_store(field(72));
        s.local_get(0)
            .local_get(0)
            .i32_load(field(76))
            .local_get(1)
            .i32_add()
            .i32_store(field(76))
            .end();
        builder.push_body(commit);
        builder
    }
}

impl ModuleBuilder {
    pub(crate) fn add_region(&mut self, ir: &RegionIr) {
        2_u32.encode(&mut self.functions);
        self.push_body(compile_region(ir));
    }

    fn push_body(&mut self, body: Function) {
        let body = body.into_raw_body();
        let mut length = Vec::new();
        (body.len() as u32).encode(&mut length);
        self.code_size += length.len() + body.len();
        self.bodies.push_back(length);
        self.bodies.push_back(body);
    }

    pub(crate) fn begin_assembly(mut self, output: &mut Vec<u8>) -> Result<VecDeque<Vec<u8>>, String> {
        let regions = self.functions.len() as u32 - 3;
        4_u32.encode(&mut self.functions);
        let mut exports = ExportSection::new();
        exports.export("dispatch", ExportKind::Func, FIRST_REGION + regions);
        let mut export_bytes = vec![SectionId::Export.into()];
        exports.encode(&mut export_bytes);
        let mut dispatcher = Function::new([(1, ValType::I32)]);
        let mut s = dispatcher.instructions();
        // Warmup has no access context and returns at the frame's current PC.
        s.local_get(1).i32_eqz().if_(BlockType::Empty);
        s.local_get(0)
            .local_get(1)
            .local_get(0)
            .i32_load(field(60))
            .i32_const(0)
            .call(BOUNDARY)
            .drop();
        s.local_get(0)
            .local_get(0)
            .i32_load(field(60))
            .i32_const(0)
            .i32_const(0x1f)
            .call(ENTRY)
            .drop();
        s.local_get(0).i32_const(0).call(COMMIT).end();
        if regions == 0 {
            s.i32_const(CompiledExit::End as i32);
        } else {
            s.loop_(BlockType::Empty);
            s.local_get(0).local_get(1).local_get(2).call_indirect(0, 2).local_tee(3);
            s.i32_const(CompiledExit::Dispatch as i32).i32_ne().if_(BlockType::Empty);
            s.local_get(3).return_().end();
            s.local_get(1)
                .local_get(0)
                .i32_load(field(60))
                .local_get(0)
                .i32_load(field(64))
                .call(RESOLVE);
            s.local_tee(2).i32_const(-1).i32_eq().if_(BlockType::Empty);
            s.i32_const(CompiledExit::Dispatch as i32).return_().end();
            s.br(0).end().unreachable();
        }
        s.end();
        self.push_body(dispatcher);
        let mut module = Module::new();
        let mut types = TypeSection::new();
        types.ty().function([ValType::I32; 4], [ValType::I32]);
        types.ty().function([ValType::I32; 3], []);
        types.ty().function([ValType::I32; 2], [ValType::I32]);
        types.ty().function([ValType::I32; 3], [ValType::I64]);
        types.ty().function([ValType::I32; 3], [ValType::I32]);
        types.ty().function([ValType::I32; 2], []);
        types.ty().function([ValType::I32], [ValType::I32]);
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
        imports.import("wie", "pages", EntityType::Function(6));
        imports.import("wie", "sample_prepare", EntityType::Function(1));
        imports.import("wie", "word_range", EntityType::Function(3));
        imports.import("wie", "resolve", EntityType::Function(4));
        module.section(&imports);
        let prefix = module.finish();
        let mut tables = TableSection::new();
        tables.table(TableType {
            element_type: RefType::FUNCREF,
            minimum: u64::from(regions),
            maximum: None,
            table64: false,
            shared: false,
        });
        let mut table_bytes = vec![SectionId::Table.into()];
        tables.encode(&mut table_bytes);
        let mut elements = ElementSection::new();
        elements.active(
            None,
            &ConstExpr::i32_const(0),
            Elements::Functions(Cow::Owned((FIRST_REGION..FIRST_REGION + regions).collect())),
        );
        let mut element_bytes = vec![SectionId::Element.into()];
        elements.encode(&mut element_bytes);
        let count = self.functions.len() as u32;
        let [functions_header, code_header] = [
            (SectionId::Function, self.functions.len(), count),
            (SectionId::Code, self.code_size, count),
        ]
        .map(|(id, length, count)| {
            let mut count_bytes = Vec::new();
            count.encode(&mut count_bytes);
            let mut header = vec![id.into()];
            let size = u32::try_from(length + count_bytes.len()).map_err(|_| String::from("Wasm section exceeds 4 GiB"))?;
            size.encode(&mut header);
            header.extend_from_slice(&count_bytes);
            Ok::<_, String>(header)
        });
        let (functions_header, code_header) = (functions_header?, code_header?);
        // Reserve once; subsequent assembly copies at most one chunk and never reallocates the output.
        output.reserve_exact(
            prefix.len()
                + self.functions.len()
                + export_bytes.len()
                + self.code_size
                + functions_header.len()
                + code_header.len()
                + table_bytes.len()
                + element_bytes.len(),
        );
        self.bodies.push_front(code_header);
        self.bodies.push_front(element_bytes);
        self.bodies.push_front(export_bytes);
        self.bodies.push_front(table_bytes);
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
    s.local_get(0).i32_load(field(64)).local_set(CPSR);
    s.local_get(0).i32_load(field(68)).local_set(END);
    s.local_get(0).i32_load(field(72)).i32_const(1).i32_sub().local_set(SAMPLE_AT);
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
            let fallthrough = instruction.pc.wrapping_add(u32::from(instruction.size)) as i32;
            let writes_pc = instruction.operation.writes_pc();
            if writes_pc {
                s.i32_const(fallthrough).local_set(NEXT_PC);
            }
            let conditional = instruction.condition != Condition::Always;
            if conditional {
                condition(&mut s, instruction.condition);
                s.if_(BlockType::Empty);
            }
            operation(&mut s, instruction, ir.entry.thumb, exit_depth + u32::from(conditional));
            if conditional {
                s.end();
            }
            s.local_get(0);
            if writes_pc {
                s.local_get(NEXT_PC);
            } else {
                s.i32_const(fallthrough);
            }
            s.i32_store(field(60));
            s.local_get(EXECUTED).i32_const(1).i32_add().local_set(EXECUTED);
        }
        s.br(count - index as u32);
    }
    s.end().i32_const(CompiledExit::Dispatch as i32).br(1).end();
    s.unreachable().end();
    // Every logical exit commits the completed prefix once; the exit reason stays on the stack.
    s.local_get(0).local_get(EXECUTED).call(COMMIT);
    s.end();
    function
}

fn boundaries(s: &mut InstructionSink<'_>, ir: &RegionIr, instruction_pc: Option<u32>, exit_depth: u32) {
    if let Some(pc) = instruction_pc {
        s.local_get(EXECUTED).local_get(SAMPLE_AT).i32_gt_u();
        s.i32_const(pc as i32).local_get(END).i32_eq().i32_or().if_(BlockType::Empty);
        s.local_get(0)
            .local_get(1)
            .i32_const(pc as i32)
            .local_get(EXECUTED)
            .call(BOUNDARY)
            .br(exit_depth + 1)
            .end();
        s.local_get(EXECUTED).local_get(SAMPLE_AT).i32_eq().if_(BlockType::Empty);
        commit_prefix(s);
        s.i32_const(0).local_set(PAGE_TABLE);
        s.local_get(1).i32_const(pc as i32).local_get(0).i32_load(field(28)).call(SAMPLE).end();
    } else {
        s.local_get(0).i32_load(field(60)).local_set(PC);
        s.local_get(0)
            .local_get(PC)
            .local_get(EXECUTED)
            .i32_const(i32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 })
            .call(ENTRY)
            .local_tee(ACCESS_STATUS)
            .local_get(ACCESS_STATUS)
            .i32_const(-1)
            .i32_ne()
            .br_if(exit_depth)
            .drop();
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

fn operand(s: &mut InstructionSink<'_>, operand: Operand, pc: u32, thumb: bool, set_carry: bool) {
    if let Some((result, carry)) = constant_operand(operand) {
        s.i32_const(result as i32).local_set(RIGHT);
        if set_carry {
            if let Some(carry) = carry {
                s.i32_const(carry as i32);
            } else {
                flag(s, 29);
            }
            s.local_set(CARRY);
        }
        return;
    }
    value(s, operand.value, pc, thumb);
    s.local_set(RIGHT);
    if operand.shift == Shift::Rrx {
        s.local_get(RIGHT).i32_const(1).i32_shr_u();
        flag(s, 29);
        s.i32_const(31).i32_shl().i32_or();
        if set_carry {
            s.local_get(RIGHT).i32_const(1).i32_and().local_set(CARRY);
        }
        s.local_set(RIGHT);
        return;
    }
    let reg = match operand.amount {
        ShiftAmount::Immediate(amount) => {
            if amount == 0 {
                if set_carry {
                    flag(s, 29);
                    s.local_set(CARRY);
                }
                return;
            }
            let amount = i32::from(amount);
            match operand.shift {
                Shift::Lsl | Shift::Lsr => {
                    if set_carry {
                        if amount <= 32 {
                            s.local_get(RIGHT)
                                .i32_const(if operand.shift == Shift::Lsl { 32 - amount } else { amount - 1 })
                                .i32_shr_u()
                                .i32_const(1)
                                .i32_and();
                        } else {
                            s.i32_const(0);
                        }
                        s.local_set(CARRY);
                    }
                    if amount < 32 {
                        s.local_get(RIGHT).i32_const(amount);
                        if operand.shift == Shift::Lsl {
                            s.i32_shl();
                        } else {
                            s.i32_shr_u();
                        }
                    } else {
                        s.i32_const(0);
                    }
                }
                Shift::Asr => {
                    if set_carry {
                        s.local_get(RIGHT)
                            .i32_const((amount - 1).min(31))
                            .i32_shr_u()
                            .i32_const(1)
                            .i32_and()
                            .local_set(CARRY);
                    }
                    s.local_get(RIGHT).i32_const(amount.min(31)).i32_shr_s();
                }
                Shift::Ror => {
                    s.local_get(RIGHT).i32_const(amount).i32_rotr();
                    if set_carry {
                        s.local_tee(RIGHT).local_get(RIGHT).i32_const(31).i32_shr_u().local_set(CARRY);
                    }
                }
                Shift::Rrx => {}
            }
            s.local_set(RIGHT);
            return;
        }
        ShiftAmount::Register(reg) => reg,
    };
    if set_carry {
        flag(s, 29);
        s.local_set(CARRY);
    }
    value(s, Value::Register(reg), pc, thumb);
    s.i32_const(255).i32_and();
    s.local_tee(AMOUNT).if_(BlockType::Empty);
    match operand.shift {
        Shift::Lsl | Shift::Lsr => {
            // Wasm masks shift counts; ARM distinguishes zero, exactly 32, and larger counts.
            s.local_get(AMOUNT).i32_const(32).i32_le_u().if_(BlockType::Empty);
            if set_carry {
                s.local_get(RIGHT);
                if operand.shift == Shift::Lsl {
                    s.i32_const(32).local_get(AMOUNT).i32_sub();
                } else {
                    s.local_get(AMOUNT).i32_const(1).i32_sub();
                }
                s.i32_shr_u().i32_const(1).i32_and().local_set(CARRY);
            }
            s.local_get(RIGHT).local_get(AMOUNT);
            if operand.shift == Shift::Lsl {
                s.i32_shl();
            } else {
                s.i32_shr_u();
            }
            s.i32_const(0).local_get(AMOUNT).i32_const(32).i32_lt_u().select().local_set(RIGHT);
            s.else_().i32_const(0).local_set(RIGHT);
            if set_carry {
                s.i32_const(0).local_set(CARRY);
            }
            s.end();
        }
        Shift::Asr => {
            s.local_get(AMOUNT)
                .i32_const(32)
                .local_get(AMOUNT)
                .i32_const(32)
                .i32_lt_u()
                .select()
                .local_set(AMOUNT);
            if set_carry {
                s.local_get(RIGHT)
                    .local_get(AMOUNT)
                    .i32_const(1)
                    .i32_sub()
                    .i32_shr_u()
                    .i32_const(1)
                    .i32_and()
                    .local_set(CARRY);
            }
            s.local_get(RIGHT)
                .i64_extend_i32_s()
                .local_get(AMOUNT)
                .i64_extend_i32_u()
                .i64_shr_s()
                .i32_wrap_i64()
                .local_set(RIGHT);
        }
        Shift::Ror => {
            s.local_get(RIGHT).local_get(AMOUNT).i32_rotr().local_set(RIGHT);
            if set_carry {
                s.local_get(RIGHT).i32_const(31).i32_shr_u().local_set(CARRY);
            }
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
        Condition::Eq | Condition::Ne | Condition::Cs | Condition::Cc | Condition::Mi | Condition::Pl | Condition::Vs | Condition::Vc => {
            let shift = match condition {
                Condition::Eq | Condition::Ne => 1,
                Condition::Cs | Condition::Cc => 2,
                Condition::Vs | Condition::Vc => 3,
                _ => 0,
            };
            s.local_get(CPSR);
            if shift != 0 {
                s.i32_const(shift).i32_shl();
            }
            s.i32_const(0).i32_lt_s();
        }
        Condition::Hi | Condition::Ls => {
            s.local_get(CPSR).i32_const(0x6000_0000).i32_and().i32_const(0x2000_0000).i32_eq();
        }
        Condition::Ge | Condition::Lt | Condition::Gt | Condition::Le => {
            // Move N xor V, and Z for strict comparisons, into the sign bit.
            s.local_get(CPSR).local_get(CPSR).i32_const(3).i32_shl().i32_xor();
            if matches!(condition, Condition::Gt | Condition::Le) {
                s.local_get(CPSR).i32_const(1).i32_shl().i32_or();
            }
            s.i32_const(0).i32_ge_s();
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
    operand(s, address.offset, pc, thumb, false);
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

// Host calls can fail; publish the completed prefix before crossing that boundary.
fn commit_prefix(s: &mut InstructionSink<'_>) {
    s.local_get(0).local_get(EXECUTED).call(COMMIT);
    s.local_get(SAMPLE_AT).local_get(EXECUTED).i32_sub().local_set(SAMPLE_AT);
    s.i32_const(0).local_set(EXECUTED);
}

fn word_range(s: &mut InstructionSink<'_>, words: u32, exit_depth: u32) {
    commit_prefix(s);
    s.i32_const(0).local_set(PAGE_TABLE);
    s.local_get(1)
        .local_get(LEFT)
        .i32_const(words as i32)
        .call(WORD_RANGE)
        .local_tee(WIDE)
        .i64_eqz()
        .if_(BlockType::Empty);
    s.i32_const(CompiledExit::InterpretOne as i32).br(exit_depth + 1).end();
    s.local_get(WIDE).i32_wrap_i64().local_set(RANGE_FIRST);
    s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64().local_set(RANGE_SECOND);
    s.local_get(0).i32_load(field(84)).local_set(RANGE_LENGTH);
}

fn scalar_address(s: &mut InstructionSink<'_>, width: Width, exit_depth: u32) {
    let alignment_mask = match width {
        Width::Byte => 0,
        Width::Half => 1,
        Width::Word => 3,
    };
    if alignment_mask != 0 {
        s.local_get(LEFT).i32_const(alignment_mask).i32_and().if_(BlockType::Empty);
        s.i32_const(CompiledExit::InterpretOne as i32).br(exit_depth + 1).end();
    }
    s.local_get(PAGE_TABLE).i32_eqz().if_(BlockType::Empty);
    commit_prefix(s);
    s.local_get(1).call(PAGES).local_set(PAGE_TABLE).end();
    // Each repr(C) directory entry is 16 bytes; its first word is the nullable page pointer.
    s.local_get(PAGE_TABLE)
        .local_get(LEFT)
        .i32_const(16)
        .i32_shr_u()
        .i32_const(4)
        .i32_shl()
        .i32_add()
        .i32_load(field(0))
        .local_tee(PAGE_POINTER)
        .i32_eqz()
        .if_(BlockType::Empty);
    s.i32_const(CompiledExit::InterpretOne as i32).br(exit_depth + 1).end();
    s.local_get(PAGE_POINTER).local_get(LEFT).i32_const(65535).i32_and().i32_add();
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
            if (!set_flags || matches!(op, AluOp::Move | AluOp::Not))
                && let (Value::Immediate(left), Some((right, carry))) = (left, constant_operand(right))
            {
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
                    if set_flags {
                        let flags = (result & 0x8000_0000) | (u32::from(result == 0) << 30) | (carry.unwrap_or(0) << 29);
                        s.local_get(0)
                            .local_get(CPSR)
                            .i32_const(if carry.is_some() { 0x1fff_ffff } else { 0x3fff_ffff })
                            .i32_and();
                        s.i32_const(flags as i32).i32_or().local_tee(CPSR).i32_store(field(64));
                    }
                    return;
                }
            }
            if !matches!(op, AluOp::Move | AluOp::Not | AluOp::CountLeadingZeros) {
                value(s, left, instruction.pc, thumb);
                s.local_set(LEFT);
            }
            let arithmetic = matches!(
                op,
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry
            );
            let zero_arithmetic = matches!(op, AluOp::Add | AluOp::Sub) && constant_operand(right).is_some_and(|(value, _)| value == 0);
            let preserves_carry = right.amount == ShiftAmount::Immediate(0) && right.shift != Shift::Rrx;
            if !zero_arithmetic {
                operand(
                    s,
                    right,
                    instruction.pc,
                    thumb,
                    set_flags && !arithmetic && op != AluOp::Multiply && !preserves_carry,
                );
            }
            match op {
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry => {
                    if matches!(op, AluOp::ReverseSub | AluOp::ReverseSubCarry) {
                        s.local_get(LEFT).local_get(RIGHT).local_set(LEFT).local_set(RIGHT);
                    }
                    let subtract = !matches!(op, AluOp::Add | AluOp::AddCarry);
                    if zero_arithmetic {
                        s.local_get(LEFT);
                    } else if !set_flags {
                        s.local_get(LEFT).local_get(RIGHT);
                        if subtract {
                            s.i32_sub();
                        } else {
                            s.i32_add();
                        }
                        if matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) {
                            flag(s, 29);
                            s.i32_add();
                            if subtract {
                                s.i32_const(1).i32_sub();
                            }
                        }
                    } else if matches!(op, AluOp::Add | AluOp::Sub | AluOp::ReverseSub) {
                        s.local_get(LEFT).local_get(RIGHT);
                        if subtract {
                            s.i32_sub().local_set(RESULT);
                            s.local_get(LEFT).local_get(RIGHT).i32_ge_u().local_set(CARRY);
                            // Keep the shared overflow calculation's normalized addends.
                            s.local_get(RIGHT).i32_const(-1).i32_xor().local_set(RIGHT);
                        } else {
                            s.i32_add().local_set(RESULT);
                            s.local_get(RESULT).local_get(LEFT).i32_lt_u().local_set(CARRY);
                        }
                        s.local_get(RESULT);
                    } else {
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
                    } else if op == AluOp::Multiply || preserves_carry {
                        0x3fff_ffff
                    } else {
                        0x1fff_ffff
                    })
                    .i32_and();
                s.local_get(RESULT).i32_const(i32::MIN).i32_and().i32_or();
                s.local_get(RESULT).i32_eqz().i32_const(30).i32_shl().i32_or();
                if zero_arithmetic {
                    if op == AluOp::Sub {
                        s.i32_const(1 << 29).i32_or();
                    }
                } else if arithmetic {
                    if matches!(op, AluOp::Add | AluOp::Sub | AluOp::ReverseSub) {
                        s.local_get(CARRY);
                    } else {
                        s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64();
                    }
                    s.i32_const(29).i32_shl().i32_or();
                    // Overflow uses the normalized addends, including the complemented subtrahend.
                    s.local_get(LEFT).local_get(RIGHT).i32_xor().i32_const(-1).i32_xor();
                    s.local_get(LEFT).local_get(RESULT).i32_xor().i32_and();
                    s.i32_const(31).i32_shr_u().i32_const(28).i32_shl().i32_or();
                } else if op != AluOp::Multiply && !preserves_carry {
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
            scalar_address(s, width, exit_depth);
            if let Operation::Store { value: source, .. } = instruction.operation {
                if source == Value::Register(15) {
                    s.i32_const(instruction.pc.wrapping_add(12) as i32);
                } else {
                    value(s, source, instruction.pc, thumb);
                }
                match width {
                    Width::Byte => {
                        s.i32_store8(MemArg { align: 0, ..field(0) });
                    }
                    Width::Half => {
                        s.i32_store16(MemArg { align: 1, ..field(0) });
                    }
                    Width::Word => {
                        s.i32_store(field(0));
                    }
                }
            } else if let Operation::Load { destination, signed, .. } = instruction.operation {
                match (width, signed) {
                    (Width::Byte, false) => {
                        s.i32_load8_u(MemArg { align: 0, ..field(0) });
                    }
                    (Width::Byte, true) => {
                        s.i32_load8_s(MemArg { align: 0, ..field(0) });
                    }
                    (Width::Half, false) => {
                        s.i32_load16_u(MemArg { align: 1, ..field(0) });
                    }
                    (Width::Half, true) => {
                        s.i32_load16_s(MemArg { align: 1, ..field(0) });
                    }
                    (Width::Word, _) => {
                        s.i32_load(field(0));
                    }
                }
                s.local_set(RIGHT);
                if destination != 15 {
                    s.local_get(0);
                }
                s.local_get(RIGHT);
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
            scalar_address(s, width, exit_depth);
            s.local_set(RANGE_FIRST);
            if width == Width::Word {
                s.local_get(RANGE_FIRST).i32_load(field(0)).local_set(RESULT);
                s.local_get(RANGE_FIRST).local_get(RIGHT).i32_store(field(0));
            } else {
                s.local_get(RANGE_FIRST).i32_load8_u(MemArg { align: 0, ..field(0) }).local_set(RESULT);
                s.local_get(RANGE_FIRST).local_get(RIGHT).i32_store8(MemArg { align: 0, ..field(0) });
            }
            s.local_get(0).local_get(RESULT).i32_store(field(u64::from(destination) * 4));
        }
        Operation::Nop => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};

    use wasm_encoder::FunctionSection;
    use wie_arm_jit_types::{CodePageStamp, CompileRegion, CompileRequest, RegionKey, ir::BasicBlock};

    use crate::Compiler;

    use super::*;

    #[test]
    fn conditions_emit_direct_bit_comparisons() {
        for (positive, negative, bytes) in [
            (Condition::Eq, Condition::Ne, vec![0x20, 6, 0x41, 1, 0x74, 0x41, 0, 0x48]),
            (Condition::Cs, Condition::Cc, vec![0x20, 6, 0x41, 2, 0x74, 0x41, 0, 0x48]),
            (Condition::Mi, Condition::Pl, vec![0x20, 6, 0x41, 0, 0x48]),
            (Condition::Vs, Condition::Vc, vec![0x20, 6, 0x41, 3, 0x74, 0x41, 0, 0x48]),
            (
                Condition::Hi,
                Condition::Ls,
                vec![0x20, 6, 0x41, 0x80, 0x80, 0x80, 0x80, 6, 0x71, 0x41, 0x80, 0x80, 0x80, 0x80, 2, 0x46],
            ),
            (Condition::Ge, Condition::Lt, vec![0x20, 6, 0x20, 6, 0x41, 3, 0x74, 0x73, 0x41, 0, 0x4e]),
            (
                Condition::Gt,
                Condition::Le,
                vec![0x20, 6, 0x20, 6, 0x41, 3, 0x74, 0x73, 0x20, 6, 0x41, 1, 0x74, 0x72, 0x41, 0, 0x4e],
            ),
        ] {
            for code in [positive, negative] {
                let mut function = Function::new([]);
                condition(&mut function.instructions(), code);
                let mut expected = vec![0];
                expected.extend_from_slice(&bytes);
                if code == negative {
                    expected.push(0x45);
                }
                assert_eq!(function.into_raw_body(), expected, "{code:?}");
            }
        }
    }

    #[test]
    fn dispatcher_is_the_only_export_and_reaches_all_region_slots() {
        let ir = RegionIr {
            entry: RegionKey {
                pc: 0x1000,
                thumb: false,
                cpu_mode: 0x1f,
            },
            blocks: vec![BasicBlock {
                instructions: vec![Instruction {
                    pc: 0x1000,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Nop,
                }],
            }],
        };
        for regions in [0, 1, 130] {
            let mut builder = ModuleBuilder::default();
            let mut functions = FunctionSection::new();
            let mut exports = ExportSection::new();
            functions.function(0);
            functions.function(0);
            functions.function(5);
            for _ in 0..regions {
                builder.add_region(&ir);
                functions.function(2);
            }
            functions.function(4);
            exports.export("dispatch", ExportKind::Func, FIRST_REGION + regions);
            let chunks = builder.begin_assembly(&mut Vec::new()).unwrap();
            let mut expected = vec![SectionId::Function.into()];
            functions.encode(&mut expected);
            assert_eq!([chunks[1].as_slice(), chunks[2].as_slice()].concat(), expected);
            let mut expected = vec![SectionId::Export.into()];
            exports.encode(&mut expected);
            assert_eq!(chunks[4], expected);
            let mut elements = ElementSection::new();
            elements.active(
                None,
                &ConstExpr::i32_const(0),
                Elements::Functions(Cow::Owned((FIRST_REGION..FIRST_REGION + regions).collect())),
            );
            let mut expected = vec![SectionId::Element.into()];
            elements.encode(&mut expected);
            assert_eq!(chunks[5], expected);
        }
    }

    #[test]
    fn large_batches_compile_into_a_single_module() {
        let request = CompileRequest {
            images: Arc::from([]),
            regions: Box::new((0..16).map(|step| {
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
                            instructions: (0..512)
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
                    source_bytes: vec![],
                })
            })),
        };
        let mut compiler = Compiler::new(request);
        assert!(!compiler.step().unwrap());
        assert!(compiler.artifact.manifest.is_empty());
        assert!(!compiler.step().unwrap());
        assert!(compiler.artifact.manifest.is_empty());
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
        assert_eq!(
            artifact.manifest.iter().map(|region| region.instruction_pcs.len()).sum::<usize>(),
            8 * 512
        );
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
                true,
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
    fn register_immediate_shifts_emit_only_the_known_operation_without_unused_carry() {
        for shift in [Shift::Lsl, Shift::Lsr, Shift::Asr, Shift::Ror] {
            for amount in [0, 1, 31, 32, 33, 255] {
                let mut actual = Function::new([]);
                operand(
                    &mut actual.instructions(),
                    Operand {
                        value: Value::Register(1),
                        shift,
                        amount: ShiftAmount::Immediate(amount),
                    },
                    0x1000,
                    false,
                    false,
                );
                let mut expected = Function::new([]);
                let mut s = expected.instructions();
                s.local_get(0).i32_load(field(4)).local_set(RIGHT);
                if amount != 0 {
                    if matches!(shift, Shift::Lsl | Shift::Lsr) && amount >= 32 {
                        s.i32_const(0);
                    } else {
                        s.local_get(RIGHT)
                            .i32_const(i32::from(if shift == Shift::Asr { amount.min(31) } else { amount }));
                        match shift {
                            Shift::Lsl => {
                                s.i32_shl();
                            }
                            Shift::Lsr => {
                                s.i32_shr_u();
                            }
                            Shift::Asr => {
                                s.i32_shr_s();
                            }
                            Shift::Ror => {
                                s.i32_rotr();
                            }
                            Shift::Rrx => unreachable!(),
                        }
                    }
                    s.local_set(RIGHT);
                }
                assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{shift:?} by {amount}");
            }
        }
    }

    #[test]
    fn region_boundaries_share_entry_checks_and_commit_the_completed_prefix() {
        for thumb in [false, true] {
            let ir = RegionIr {
                entry: RegionKey {
                    pc: 0x1000,
                    thumb,
                    cpu_mode: 0x1f,
                },
                blocks: vec![BasicBlock {
                    instructions: vec![Instruction {
                        pc: 0x1000,
                        size: if thumb { 2 } else { 4 },
                        condition: Condition::Always,
                        operation: Operation::Nop,
                    }],
                }],
            };
            for depth in [1, 129] {
                let mut actual = Function::new([]);
                boundaries(&mut actual.instructions(), &ir, None, depth);
                let mut expected = Function::new([]);
                expected
                    .instructions()
                    .local_get(0)
                    .i32_load(field(60))
                    .local_set(PC)
                    .local_get(0)
                    .local_get(PC)
                    .local_get(EXECUTED)
                    .i32_const(if thumb { 0x3f } else { 0x1f })
                    .call(ENTRY)
                    .local_tee(ACCESS_STATUS)
                    .local_get(ACCESS_STATUS)
                    .i32_const(-1)
                    .i32_ne()
                    .br_if(depth)
                    .drop();
                assert_eq!(actual.into_raw_body(), expected.into_raw_body());
            }
            let body = compile_region(&ir).into_raw_body();
            let mut commit = Function::new([]);
            commit.instructions().local_get(0).local_get(EXECUTED).call(COMMIT).end();
            assert!(body.ends_with(&commit.into_raw_body()[1..]));
        }
    }

    #[test]
    fn instruction_boundaries_branch_directly_with_the_exit_result() {
        let ir = RegionIr {
            entry: RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            },
            blocks: vec![],
        };
        for depth in [0, 1, 129] {
            let mut actual = Function::new([]);
            boundaries(&mut actual.instructions(), &ir, Some(0x1000), depth);
            let mut expected = Function::new([]);
            expected
                .instructions()
                .local_get(EXECUTED)
                .local_get(SAMPLE_AT)
                .i32_gt_u()
                .i32_const(0x1000)
                .local_get(END)
                .i32_eq()
                .i32_or()
                .if_(BlockType::Empty)
                .local_get(0)
                .local_get(1)
                .i32_const(0x1000)
                .local_get(EXECUTED)
                .call(BOUNDARY)
                .br(depth + 1)
                .end();
            let actual = actual.into_raw_body();
            assert!(actual.starts_with(&expected.into_raw_body()));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    #[ignore = "requires Node.js to execute generated Wasm"]
    fn generated_execution_preserves_memory_flags_and_stop_boundaries() {
        extern crate std;
        use std::{
            io::Write,
            process::{Command, Stdio},
        };

        let address = Address {
            base: Value::Register(1),
            offset: Operand {
                value: Value::Immediate(0),
                shift: Shift::Lsl,
                amount: ShiftAmount::Immediate(0),
            },
            subtract: false,
            pre_index: true,
            write_back: None,
        };
        let operations = [
            Operation::Store {
                address,
                width: Width::Word,
                value: Value::Register(0),
            },
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(0),
                left: Value::Register(0),
                right: Operand {
                    value: Value::Immediate(1),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0),
                },
                set_flags: true,
            },
            Operation::Store {
                address: Address {
                    base: Value::Register(2),
                    ..address
                },
                width: Width::Word,
                value: Value::Register(0),
            },
            Operation::Nop,
        ];
        let ir = RegionIr {
            entry: RegionKey {
                pc: 0x1000,
                thumb: true,
                cpu_mode: 0x1f,
            },
            blocks: vec![BasicBlock {
                instructions: operations
                    .into_iter()
                    .enumerate()
                    .map(|(index, operation)| Instruction {
                        pc: 0x1000 + index as u32 * 2,
                        size: 2,
                        condition: Condition::Always,
                        operation,
                    })
                    .collect(),
            }],
        };
        let mut builder = ModuleBuilder::default();
        builder.add_region(&ir);
        let mut accesses = ir.clone();
        accesses.blocks[0].instructions.clear();
        for (index, width) in [Width::Byte, Width::Half, Width::Word].into_iter().enumerate() {
            for operation in [
                Operation::Store {
                    address,
                    width,
                    value: Value::Register(0),
                },
                Operation::Load {
                    address,
                    width,
                    signed: false,
                    destination: 2 + index as u8 * 2,
                },
                Operation::Load {
                    address,
                    width,
                    signed: true,
                    destination: 3 + index as u8 * 2,
                },
            ] {
                let pc = 0x1000 + accesses.blocks[0].instructions.len() as u32 * 2;
                accesses.blocks[0].instructions.push(Instruction {
                    pc,
                    size: 2,
                    condition: Condition::Always,
                    operation,
                });
            }
        }
        builder.add_region(&accesses);
        for (index, op) in [
            AluOp::Add,
            AluOp::Sub,
            AluOp::ReverseSub,
            AluOp::AddCarry,
            AluOp::SubCarry,
            AluOp::ReverseSubCarry,
            AluOp::Add,
            AluOp::Sub,
            AluOp::Move,
            AluOp::Not,
            AluOp::And,
            AluOp::Or,
            AluOp::Xor,
            AluOp::BitClear,
        ]
        .into_iter()
        .enumerate()
        {
            builder.add_region(&RegionIr {
                entry: ir.entry,
                blocks: vec![BasicBlock {
                    instructions: vec![Instruction {
                        pc: 0x1000,
                        size: 2,
                        condition: Condition::Always,
                        operation: Operation::Alu {
                            op,
                            destination: Some(2),
                            left: Value::Register(0),
                            right: Operand {
                                value: if matches!(index, 6 | 7) {
                                    Value::Immediate(0)
                                } else {
                                    Value::Register(1)
                                },
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            set_flags: true,
                        },
                    }],
                }],
            });
        }
        for width in [Width::Byte, Width::Word] {
            builder.add_region(&RegionIr {
                entry: ir.entry,
                blocks: vec![BasicBlock {
                    instructions: vec![Instruction {
                        pc: 0x1000,
                        size: 2,
                        condition: Condition::Always,
                        operation: Operation::Swap {
                            destination: 2,
                            address: 0,
                            value: 1,
                            width,
                        },
                    }],
                }],
            });
        }
        let mut bytes = Vec::new();
        for chunk in builder.begin_assembly(&mut bytes).unwrap() {
            bytes.extend_from_slice(&chunk);
        }
        let mut node = Command::new("node")
            .args([
                "-e",
                r#"
const assert = require('node:assert/strict');
const module = new WebAssembly.Module(require('node:fs').readFileSync(0));
function directory(memory, entries) {
    const base = memory.buffer.byteLength;
    memory.grow(16);
    const view = new DataView(memory.buffer);
    for (const [page, pointer] of entries) view.setUint32(base + page * 16, pointer, true);
    return base;
}
{
    const memory = new WebAssembly.Memory({initial: 1});
    const frame = new Uint32Array(memory.buffer, 0, 22);
    frame[15] = frame[17] = 0x1000; frame[16] = 0x1f; frame[18] = 1;
    const before = Array.from(frame);
    const unexpected = () => { throw new Error('guest execution during warmup'); };
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory, pages: unexpected, word_range: unexpected, sample_prepare: unexpected, resolve: unexpected,
    }}).exports.dispatch;
    for (let slot = 0; slot < 18; slot++) assert.equal(dispatch(0, 0, slot), 3);
    assert.deepEqual(Array.from(frame), before);
}
for (const failure of ['pages', 'sample_prepare', 'resolve', 'sample', 'page_switch', null]) {
    const memory = new WebAssembly.Memory({initial: 3});
    const table = directory(memory, [[0, 65536], [1, 131072]]);
    const frame = new Uint32Array(memory.buffer, 0, 22);
    frame[0] = 42; frame[1] = 0x3000; frame[15] = 0x1000;
    frame[2] = failure === 'page_switch' ? 0x13000 : 0x3000;
    frame[16] = 0xf000003f; frame[17] = 0x2000;
    const samples = frame[18] = ['pages', 'sample_prepare', 'sample'].includes(failure) ? 3 : 1024;
    let pages = 0;
    const injected = new Error('injected host failure');
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory,
        word_range() { throw new Error('unexpected word range'); },
        pages() {
            if (failure === 'pages' && pages === 1) throw injected;
            pages++;
            return table;
        },
        sample_prepare() { if (failure === 'sample_prepare') throw injected; },
        resolve() { if (failure === 'resolve') throw injected; return -1; },
    }}).exports.dispatch;
    if (failure === null || failure === 'sample' || failure === 'page_switch') {
        assert.equal(dispatch(0, 1, 0), failure === 'sample' ? 1 : 0);
    } else {
        assert.throws(() => dispatch(0, 1, 0), error => error === injected);
    }
    const completed = failure === 'sample' ? 3 : ['resolve', 'page_switch', null].includes(failure) ? 4 : 2;
    assert.equal(frame[0], 43);
    assert.equal(frame[15], 0x1000 + completed * 2);
    assert.equal(frame[16], 0x3f);
    assert.equal(frame[18], samples - completed);
    assert.equal(frame[19], completed);
    assert.equal(pages, failure === 'sample' ? 2 : 1);
    assert.equal(new DataView(memory.buffer).getUint32(0x13000, true), completed === 2 || failure === 'page_switch' ? 42 : 43);
    if (failure === 'page_switch') assert.equal(new DataView(memory.buffer).getUint32(0x23000, true), 43);
}
for (let sample = 1; sample <= 5; sample++) for (let end = 0; end <= 5; end++) {
    const memory = new WebAssembly.Memory({initial: 2});
    const table = directory(memory, [[0, 65536]]);
    const frame = new Uint32Array(memory.buffer, 0, 22);
    frame[0] = 42; frame[1] = frame[2] = 0x3000;
    frame[15] = 0x1000; frame[16] = 0xf000003f;
    frame[17] = 0x1000 + end * 2; frame[18] = sample;
    let samples = 0;
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory,
        word_range() { throw new Error('unexpected word range'); },
        resolve() { return -1; },
        pages() { return table; },
        sample_prepare() { samples++; },
    }}).exports.dispatch;
    const completed = Math.min(4, sample, end);
    const exit = completed === end ? 3 : completed === sample ? 1 : 0;
    assert.equal(dispatch(0, 1, 0), exit, `sample=${sample}, end=${end}`);
    assert.equal(frame[15], 0x1000 + completed * 2);
    assert.equal(frame[16], completed < 2 ? 0xf000003f : 0x3f);
    assert.equal(frame[18], sample - completed);
    assert.equal(frame[19], completed);
    assert.equal(samples, completed === sample ? 1 : 0);
    assert.equal(new DataView(memory.buffer).getUint32(0x13000, true), completed === 0 ? 0 : completed < 3 ? 42 : 43);
}
for (const [index, width] of [1, 4].entries()) for (const address of [0x3000, 0x3001, 0xfffc, 0xffff, 0xfffffffc, null]) {
    const memory = new WebAssembly.Memory({initial: 2});
    const table = directory(memory, address === null ? [] : [[address >>> 16, 65536]]);
    const frame = new Uint32Array(memory.buffer, 0, 22);
    const data = new Uint8Array(memory.buffer, 65536, 65536).fill(0x55);
    frame[0] = address ?? 0x3000; frame[1] = 0x89abcdef;
    frame[15] = 0x1000; frame[16] = 0xf000003f; frame[17] = 0x1002;
    frame[18] = 1024;
    const unexpected = () => { throw new Error('unexpected host call'); };
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory, word_range: unexpected, sample_prepare: unexpected, resolve: unexpected,
        pages() { return table; },
    }}).exports.dispatch;
    const admitted = address !== null && address % width === 0;
    assert.equal(dispatch(0, 1, 16 + index), admitted ? 3 : 4);
    assert.equal(frame[2], admitted ? width === 1 ? 0x55 : 0x55555555 : 0);
    assert.equal(frame[16], 0xf000003f);
    assert.equal(frame[19], admitted ? 1 : 0);
    if (admitted) {
        assert.deepEqual(Array.from(data.slice(address & 65535, (address & 65535) + width)), [0xef, 0xcd, 0xab, 0x89].slice(0, width));
    } else {
        assert(data.every(byte => byte === 0x55));
    }
}
for (const address of [0x3000, 0xfffc, 0xfffffffc, 0x3001, null]) {
    const memory = new WebAssembly.Memory({initial: 2});
    const table = directory(memory, address === null ? [] : [[address >>> 16, 65536]]);
    const frame = new Uint32Array(memory.buffer, 0, 22);
    frame[0] = 0x89abcdef; frame[1] = address ?? 0x3000;
    frame[15] = 0x1000; frame[16] = 0x3f; frame[17] = 0x1012;
    frame[18] = 1024;
    let pages = 0;
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory,
        word_range() { throw new Error('unexpected word range'); },
        sample_prepare() { throw new Error('unexpected sample'); },
        resolve() { throw new Error('unexpected resolve'); },
        pages() {
            pages++;
            return table;
        },
    }}).exports.dispatch;
    const completed = address === null ? 0 : address === 0x3001 ? 3 : 9;
    assert.equal(dispatch(0, 1, 1), completed === 9 ? 3 : 4);
    assert.equal(pages, 1);
    assert.equal(frame[15], 0x1000 + completed * 2);
    assert.equal(frame[18], 1024 - completed);
    assert.equal(frame[19], completed);
    const expected = [0xef, 0xffffffef, 0xcdef, 0xffffcdef, 0x89abcdef, 0x89abcdef];
    assert.deepEqual(Array.from(frame.slice(2, 2 + completed / 3 * 2)), expected.slice(0, completed / 3 * 2));
}
{
    const memory = new WebAssembly.Memory({initial: 1});
    const frame = new Uint32Array(memory.buffer, 0, 22);
    const unexpected = () => { throw new Error('unexpected host call'); };
    const dispatch = new WebAssembly.Instance(module, {wie: {
        memory, pages: unexpected, word_range: unexpected,
        sample_prepare: unexpected, resolve: unexpected,
    }}).exports.dispatch;
    const values = [0, 1, 2, 0x7fffffff, 0x80000000, 0x80000001, 0xfffffffe, 0xffffffff];
    for (let op = 0; op < 14; op++) for (const left of values) for (const right of values) for (const carry of [0, 1]) {
        frame.fill(0);
        frame[0] = left; frame[1] = right;
        frame[15] = 0x1000; frame[16] = (0xd800003f | carry << 29) >>> 0;
        frame[17] = 0x1002; frame[18] = 1024;
        assert.equal(dispatch(0, 1, 2 + op), 3);
        if (op >= 8) {
            const result = [right, ~right, left & right, left | right, left ^ right, left & ~right][op - 8] >>> 0;
            const flags = ((result & 0x80000000) | (result === 0 ? 0x40000000 : 0) | carry << 29 | 0x10000000) >>> 0;
            assert.equal(frame[2], result);
            assert.equal(frame[16], (flags | 0x0800003f) >>> 0);
            continue;
        }
        const kind = op % 6;
        const operand = op >= 6 ? 0 : right;
        const reverse = kind === 2 || kind === 5;
        const subtract = kind !== 0 && kind !== 3;
        const a = BigInt(reverse ? operand : left), b = BigInt(reverse ? left : operand);
        const carryIn = kind >= 3 ? BigInt(carry) : subtract ? 1n : 0n;
        const wide = a + (subtract ? 0xffffffffn - b : b) + carryIn;
        const result = Number(wide & 0xffffffffn);
        const signed = BigInt.asIntN(32, a) + (subtract ? -BigInt.asIntN(32, b) : BigInt.asIntN(32, b))
            + (subtract ? carryIn - 1n : carryIn);
        const flags = ((result & 0x80000000) | (result === 0 ? 0x40000000 : 0)
            | (wide > 0xffffffffn ? 0x20000000 : 0)
            | (signed < -0x80000000n || signed > 0x7fffffffn ? 0x10000000 : 0)) >>> 0;
        assert.equal(frame[2], result, `op=${op}, left=${left}, right=${right}, carry=${carry}`);
        assert.equal(frame[16], (flags | 0x0800003f) >>> 0);
        assert.equal(frame[19], 1);
    }
}
"#,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        node.stdin.take().unwrap().write_all(&bytes).unwrap();
        let output = node.wait_with_output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }

    #[test]
    fn arithmetic_without_flag_updates_uses_wrapping_i32_operations() {
        for op in [
            AluOp::Add,
            AluOp::AddCarry,
            AluOp::Sub,
            AluOp::SubCarry,
            AluOp::ReverseSub,
            AluOp::ReverseSubCarry,
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
                        left: Value::Register(1),
                        right: Operand {
                            value: Value::Register(2),
                            shift: Shift::Lsl,
                            amount: ShiftAmount::Immediate(0),
                        },
                        set_flags: false,
                    },
                },
                false,
                0,
            );
            let mut expected = Function::new([]);
            let mut s = expected.instructions();
            s.local_get(0).i32_load(field(4)).local_set(LEFT);
            s.local_get(0).i32_load(field(8)).local_set(RIGHT);
            if matches!(op, AluOp::ReverseSub | AluOp::ReverseSubCarry) {
                s.local_get(LEFT).local_get(RIGHT).local_set(LEFT).local_set(RIGHT);
            }
            s.local_get(LEFT).local_get(RIGHT);
            let subtract = !matches!(op, AluOp::Add | AluOp::AddCarry);
            if subtract {
                s.i32_sub();
            } else {
                s.i32_add();
            }
            if matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) {
                flag(&mut s, 29);
                s.i32_add();
                if subtract {
                    s.i32_const(1).i32_sub();
                }
            }
            s.local_set(RESULT).local_get(0).local_get(RESULT).i32_store(field(0));
            assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{op:?}");
        }
    }

    #[test]
    fn constant_alu_emits_folded_results_and_move_flags() {
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
                        set_flags: matches!(op, AluOp::Move | AluOp::Not),
                    },
                },
                false,
                0,
            );
            let mut expected = Function::new([]);
            let mut s = expected.instructions();
            s.local_get(0).i32_const(result as i32).i32_store(field(0));
            if matches!(op, AluOp::Move | AluOp::Not) {
                let (mask, flags) = if op == AluOp::Move {
                    (0x1fff_ffff, 0xa000_0000_u32)
                } else {
                    (0x3fff_ffff, 0x8000_0000)
                };
                s.local_get(0).local_get(CPSR).i32_const(mask).i32_and();
                s.i32_const(flags as i32).i32_or().local_tee(CPSR).i32_store(field(64));
            }
            assert_eq!(actual.into_raw_body(), expected.into_raw_body(), "{op:?} {left:#x}, {input:#x}");
        }
    }
}
