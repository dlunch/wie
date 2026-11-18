use alloc::{
    borrow::Cow,
    collections::{BTreeMap, VecDeque, btree_map::Entry},
    string::String,
    vec,
    vec::Vec,
};

use wasm_encoder::{
    BlockType, ConstExpr, ElementSection, Elements, Encode, EntityType, ExportKind, ExportSection, Function, GlobalSection, GlobalType,
    ImportSection, InstructionSink, MemArg, MemoryType, Module, RefType, SectionId, TableSection, TableType, TypeSection, ValType,
};
use wie_core_arm_aot::CompiledExit;
use wie_core_arm_aot::ir::{
    AluOp, BasicBlock, BranchTarget, Condition, Instruction, MemoryOperand, Operand, Operation, Reg, RegionIr, Shift, ShiftAmount, Value, Width,
};

const LEFT: u32 = 2;
const RIGHT: u32 = 3;
const RESULT: u32 = 4;
const PC: u32 = 5;
const CPSR: u32 = 6;
const CARRY: u32 = 7;
const AMOUNT: u32 = 8;
const ACCESS_STATUS: u32 = 9;
const WIDE: u32 = 10;
const RANGE_FIRST: u32 = 11;
const PAGE_TABLE: u32 = 12;
const PAGE_POINTER: u32 = 13;

const NEGATIVE: u32 = PAGE_POINTER + 1;
const ZERO: u32 = NEGATIVE + 1;
const OVERFLOW: u32 = ZERO + 1;
const ORIGIN: u32 = OVERFLOW + 1;
const FIRST_REGISTER: u32 = ORIGIN + 1;
const N: u32 = 1 << 31;
const Z: u32 = 1 << 30;
const C: u32 = 1 << 29;
const V: u32 = 1 << 28;

// An immutable global.get encodes each repeated mask in two bytes.
const PAGE_MASK: i32 = 65535;

// Function indices follow the import and helper section order.
const PAGES: u32 = 0;
const WORD_RANGE: u32 = 1;
const RESOLVE: u32 = 2;
const ENTRY: u32 = 3;
const PUBLISH: u32 = 4;
const PACK_NZ: u32 = 5;
const PACK_NZC: u32 = 6;
const PACK_NZCV: u32 = 7;
type Transfers = BTreeMap<(u16, bool), u32>;

pub struct ModuleBuilder {
    functions: Vec<u8>,
    bodies: VecDeque<Vec<u8>>,
    code_size: usize,
    region_functions: Vec<u32>,
    transfers: Transfers,
}

impl Default for ModuleBuilder {
    fn default() -> Self {
        let mut builder = Self {
            functions: vec![0, 4, 0, 6, 5],
            bodies: VecDeque::new(),
            code_size: 0,
            region_functions: Vec::new(),
            transfers: Transfers::new(),
        };
        let mut entry = Function::new([]);
        let mut s = entry.instructions();
        s.local_get(1).i32_const(0x1000).i32_lt_u().if_(BlockType::Empty);
        s.local_get(0).local_get(1).i32_store(field(72));
        s.i32_const(CompiledExit::GuestFault as i32).return_().end();
        s.local_get(1).local_get(0).i32_load(field(68)).i32_eq().if_(BlockType::Empty);
        s.i32_const(CompiledExit::End as i32).return_().end();
        s.local_get(0).i32_load(field(64)).i32_const(0x0100_003f).i32_and();
        s.local_get(2).i32_ne().if_(BlockType::Empty);
        s.i32_const(CompiledExit::Dispatch as i32).return_().end();
        // T selects two-byte alignment; ARM requires four-byte alignment.
        s.local_get(1)
            .local_get(2)
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
        let mut publish = Function::new([]);
        let mut s = publish.instructions();
        s.local_get(0).local_get(1).i32_store(field(60));
        s.local_get(0).local_get(2).i32_store(field(64)).end();
        builder.push_body(publish);
        for flags in [N | Z, N | Z | C, N | Z | C | V] {
            let mut f = Function::new([]);
            let mut s = f.instructions();
            merge_flags(&mut s, flags, 0, [1, 2, 3, 4]);
            s.end();
            builder.push_body(f);
        }
        builder
    }
}

impl ModuleBuilder {
    pub fn add_region(&mut self, ir: &mut RegionIr) {
        for block in &mut ir.blocks {
            block.eliminate_dead_alu_stores();
        }
        for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
            let key = match &instruction.operation {
                Operation::MultipleTransfer { registers, load, .. } => (*registers, *load),
                Operation::DoubleTransfer { register, load, .. } => (3 << register.index(), *load),
                _ => continue,
            };
            if let Entry::Vacant(entry) = self.transfers.entry(key) {
                entry.insert(ENTRY + self.functions.len() as u32);
                self.functions.push(5);
                self.push_body(compile_transfer(key.0, key.1));
            }
        }
        self.region_functions.push(ENTRY + self.functions.len() as u32);
        self.functions.push(1);
        self.push_body(compile_region(ir, &self.transfers));
    }

    fn push_body(&mut self, body: Function) {
        let body = body.into_raw_body();
        // Bodies stay alive until the host copies the module out, so store each one exactly sized.
        let mut chunk = Vec::with_capacity(5 + body.len());
        (body.len() as u32).encode(&mut chunk);
        chunk.extend_from_slice(&body);
        self.code_size += chunk.len();
        self.bodies.push_back(chunk);
    }

    pub fn assemble(mut self) -> Result<VecDeque<Vec<u8>>, String> {
        let regions = self.region_functions.len() as u32;
        let dispatcher_index = ENTRY + self.functions.len() as u32;
        0_u32.encode(&mut self.functions);
        let mut exports = ExportSection::new();
        exports.export("dispatch", ExportKind::Func, dispatcher_index);
        let mut export_bytes = vec![SectionId::Export.into()];
        exports.encode(&mut export_bytes);
        let mut dispatcher = Function::new([(1, ValType::I32)]);
        let mut s = dispatcher.instructions();
        // Warmup has no access context and returns at the frame's current PC.
        s.local_get(1).i32_eqz().if_(BlockType::Empty);
        s.local_get(0).local_get(0).i32_load(field(60)).i32_const(0x1f).call(ENTRY).drop();
        s.end();
        if regions == 0 {
            s.i32_const(CompiledExit::End as i32);
        } else {
            s.loop_(BlockType::Empty);
            s.local_get(0).local_get(1).local_get(2).call_indirect(0, 1).local_tee(3);
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
        types.ty().function([ValType::I32; 3], [ValType::I32]);
        types.ty().function([ValType::I32; 2], [ValType::I32]);
        types.ty().function([ValType::I32; 3], [ValType::I64]);
        types.ty().function([ValType::I32], [ValType::I32]);
        types.ty().function([ValType::I32; 3], []);
        types.ty().function([ValType::I32; 5], [ValType::I32]);
        types.ty().function([ValType::I32; 4], [ValType::I32]);
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
        imports.import("wie", "pages", EntityType::Function(3));
        imports.import("wie", "word_range", EntityType::Function(2));
        imports.import("wie", "resolve", EntityType::Function(0));
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
        elements.active(None, &ConstExpr::i32_const(0), Elements::Functions(Cow::Owned(self.region_functions)));
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
        self.bodies.push_front(code_header);
        self.bodies.push_front(element_bytes);
        self.bodies.push_front(export_bytes);
        let mut globals = GlobalSection::new();
        globals.global(
            GlobalType {
                val_type: ValType::I32,
                mutable: false,
                shared: false,
            },
            &ConstExpr::i32_const(PAGE_MASK),
        );
        let mut global_bytes = vec![SectionId::Global.into()];
        globals.encode(&mut global_bytes);
        self.bodies.push_front(global_bytes);
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

// A register can stay in a local between host checkpoints, or write through to its frame slot.
#[derive(Default)]
struct RegionLocals {
    locals: [Option<u32>; 16],
    translated: [Option<u32>; 16],
    write_back: u16,
    thumb: bool,
    flag_inputs: u32,
    publish_mask: u32,
    origin: u32,
    retry_pages: bool,
    flags_written: u32,
}

impl RegionLocals {
    fn for_region(ir: &RegionIr) -> Self {
        let mut cache = Self {
            origin: ir.entry.pc,
            thumb: ir.entry.thumb,
            ..Self::default()
        };
        let mut reads = [0; 16];
        let mut writes = [0; 16];
        let mut base_uses = [0; 16];
        let mut flag_writes = 0;
        let mut checkpoints = 1; // Common region exit.
        for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
            let (written, _, read) = instruction.operation.implicit_flag_effects();
            flag_writes += u32::from(written != 0);
            cache.flags_written |= u32::from(written) << 28;
            cache.flag_inputs |= u32::from(read | instruction.condition.flag_inputs()) << 28;
            instruction
                .operation
                .visit_registers(|reg| reads[usize::from(reg.index())] += 1, |reg| writes[usize::from(reg.index())] += 1);
            match instruction.operation {
                Operation::DoubleTransfer { .. } | Operation::MultipleTransfer { .. } => checkpoints += 1,
                Operation::Load { ref address, .. } | Operation::Store { ref address, .. } => {
                    cache.retry_pages = true;
                    if let Some(index) = scalar_base(address) {
                        base_uses[index] += 1;
                    }
                }
                Operation::Swap { .. } => cache.retry_pages = true,
                _ => {}
            }
        }
        // Materialize at the smaller set of emitted sites: flag producers or consumers.
        cache.publish_mask = if flag_writes < checkpoints { 0 } else { cache.flags_written };

        // Scalar accesses share a cold page-directory retry. Pack live flags there
        // instead of charging that checkpoint to every execution of a flag producer.
        checkpoints += u32::from(cache.retry_pages);
        let mut next_local = FIRST_REGISTER;
        for index in 0..15 {
            let writes = writes[index];
            // Frame access costs five bytes; local access costs two. Each cached register needs
            // a seven-byte entry load. Each dirty register also costs seven bytes per checkpoint.
            let flush = if writes == 0 { 0 } else { 7 * checkpoints };
            if 3 * (reads[index] + writes) > 7 + flush {
                cache.locals[index] = Some(next_local);
                next_local += 1;
                if writes != 0 {
                    cache.write_back |= 1 << index;
                } else if base_uses[index] > checkpoints {
                    // Reuse an invariant address when its accesses amortize translation,
                    // including reacquisition after word-range calls.
                    cache.translated[index] = Some(next_local);
                    next_local += 1;
                }
            }
        }
        cache
    }

    fn publish(&self, s: &mut InstructionSink<'_>) {
        for (index, local) in self.locals.iter().enumerate() {
            if let Some(local) = local
                && self.write_back & (1 << index) != 0
            {
                s.local_get(0).local_get(*local).i32_store(field(index as u64 * 4));
            }
        }
    }

    fn start_store(&self, s: &mut InstructionSink<'_>, reg: &Reg) {
        if self.locals[usize::from(reg.index())].is_none() {
            s.local_get(0);
        }
    }

    fn finish_store(&self, s: &mut InstructionSink<'_>, reg: &Reg) {
        let index = usize::from(reg.index());
        if let Some(local) = self.locals[index] {
            s.local_set(local);
        } else {
            s.i32_store(field(index as u64 * 4));
        }
    }

    fn store_local(&self, s: &mut InstructionSink<'_>, reg: &Reg, value: u32) {
        self.start_store(s, reg);
        s.local_get(value);
        self.finish_store(s, reg);
    }
}

fn compile_region(ir: &RegionIr, transfers: &Transfers) -> Function {
    let instructions = || ir.blocks.iter().flat_map(|block| &block.instructions);
    let (first, last, count) = instructions().fold((u32::MAX, 0, 0), |(first, last, count), instruction| {
        let pc = instruction.pc.get();
        (first.min(pc), last.max(pc), count + 1)
    });
    let shift = if ir.entry.thumb { 1 } else { 2 };
    let mut targets = vec![count; ((last - first) >> shift) as usize + 1];
    for (index, instruction) in instructions().enumerate() {
        targets[((instruction.pc.get() - first) >> shift) as usize] = index as u32;
    }
    let regs = RegionLocals::for_region(ir);
    let extra = 2 * u32::from(regs.retry_pages);
    let register_locals = regs.locals.iter().chain(&regs.translated).flatten().count() as u32;
    let mut function = Function::new([(8, ValType::I32), (1, ValType::I64), (7 + register_locals, ValType::I32)]);
    let mut s = function.instructions();
    for (index, local) in regs.locals.iter().enumerate() {
        if let Some(local) = local {
            s.local_get(0).i32_load(field(index as u64 * 4)).local_set(*local);
        }
    }
    s.i32_const(ir.entry.pc as i32).local_set(ORIGIN);
    s.local_get(0).i32_load(field(60)).local_get(ORIGIN).i32_sub().local_set(PC);
    s.local_get(0).i32_load(field(64)).local_set(CPSR);
    unpack_flags(&mut s, regs.flags_written | regs.flag_inputs);
    s.block(BlockType::Result(ValType::I32));
    s.block(BlockType::Empty);
    s.loop_(BlockType::Empty);
    s.local_get(0)
        .local_get(PC)
        .local_get(ORIGIN)
        .i32_add()
        .i32_const(i32::from(ir.entry.cpu_mode) | if ir.entry.thumb { 0x20 } else { 0 })
        .call(ENTRY)
        .local_tee(ACCESS_STATUS)
        .local_get(ACCESS_STATUS)
        .i32_const(-1)
        .i32_ne()
        .br_if(2)
        .drop();
    // A page-directory miss retries selection at this instruction. Repeating ENTRY here
    // could stop after an earlier MSR changed the mode inside the block.
    if regs.retry_pages {
        s.loop_(BlockType::Empty).block(BlockType::Empty);
    }
    for _ in 0..=count {
        s.block(BlockType::Empty);
    }
    s.local_get(PC);
    if first != ir.entry.pc {
        s.i32_const(first.wrapping_sub(ir.entry.pc) as i32).i32_sub();
    }
    s.i32_const(shift).i32_shr_u();
    s.br_table(targets, count);
    let mut index = 0;
    for block in &ir.blocks {
        let live = block.live_flag_updates();
        s.end();
        if let Some(body) = small_loop_body(ir, block) {
            s.loop_(BlockType::Empty);
            let exit_depth = count - index + 2 + extra;
            for (i, instruction) in block.instructions.iter().enumerate() {
                emit_instruction(
                    &mut s,
                    transfers,
                    &regs,
                    instruction,
                    exit_depth,
                    i + 1 == block.instructions.len(),
                    live[i],
                );
            }
            s.local_get(PC)
                .i32_const(body.instructions[0].pc.get().wrapping_sub(ir.entry.pc) as i32)
                .i32_ne();
            s.br_if(count - index + 1 + extra);
            for (i, (instruction, flags)) in body.instructions.iter().zip(body.live_flag_updates()).enumerate() {
                emit_instruction(&mut s, transfers, &regs, instruction, exit_depth, i + 1 == body.instructions.len(), flags);
            }
            s.br(0).end();
        }
        for (instruction_index, instruction) in block.instructions.iter().enumerate() {
            if instruction_index != 0 {
                s.end();
            }
            let exit_depth = count - index + 1 + extra;
            emit_instruction(
                &mut s,
                transfers,
                &regs,
                instruction,
                exit_depth,
                instruction_index + 1 == block.instructions.len(),
                live[instruction_index],
            );
            index += 1;
        }
        s.br(count - index + 1 + extra);
    }
    s.end().i32_const(CompiledExit::Dispatch as i32).br(2 + extra).end();
    if regs.retry_pages {
        regs.publish(&mut s);
        s.local_get(0).local_get(PC).local_get(ORIGIN).i32_add();
        packed_cpsr(&mut s, regs.publish_mask);
        s.call(PUBLISH);
        s.local_get(1).call(PAGES).local_set(PAGE_TABLE);
        for (&base, &translated) in regs.locals.iter().zip(&regs.translated) {
            if let (Some(base), Some(translated)) = (base, translated) {
                load_page(&mut s, base);
                // Keep zero for an unmapped page. Only the instruction that uses it faults.
                s.local_tee(translated).if_(BlockType::Empty);
                s.local_get(translated).local_get(base).global_get(0).i32_and().i32_add();
                s.local_set(translated).end();
            }
        }
        s.br(0).end().end();
    }
    s.unreachable().end();
    s.i32_const(CompiledExit::InterpretOne as i32).end();
    regs.publish(&mut s);
    // The exit reason stays on the stack while the caller-visible frame is published.
    s.local_get(0).local_get(PC).local_get(ORIGIN).i32_add();
    packed_cpsr(&mut s, regs.publish_mask);
    s.call(PUBLISH);
    s.end();
    function
}

fn small_loop_body<'a>(ir: &'a RegionIr, header: &BasicBlock) -> Option<&'a BasicBlock> {
    let tail = header.instructions.last()?;
    let Operation::Branch {
        target: BranchTarget::Address(ref target),
        link: None,
        exchange: false,
    } = tail.operation
    else {
        return None;
    };
    if tail.condition == Condition::Always {
        return None;
    }
    let mask = if ir.entry.thumb { !1 } else { !3 };
    let successors = [target.get() & mask, tail.pc.get().wrapping_add(u32::from(tail.size))];
    let header_pc = header.instructions[0].pc.get();
    ir.blocks.iter().find(|body| {
        let body_pc = body.instructions[0].pc.get();
        let backedge = body.instructions.last().unwrap();
        // Skipping ENTRY must not bypass its low-address guest fault.
        body_pc >= 0x1000
            && successors.contains(&body_pc)
            && successors[0] != successors[1]
            && !successors.contains(&header_pc)
            && header.instructions.len() + body.instructions.len() <= 32
            && backedge.condition == Condition::Always
            && matches!(backedge.operation, Operation::Branch {
                target: BranchTarget::Address(ref pc), link: None, exchange: false
            } if pc.get() & mask == header_pc)
            && [header, body].into_iter().all(|block| {
                block.instructions[..block.instructions.len() - 1]
                    .iter()
                    .all(|instruction| !instruction.operation.writes_pc() && !matches!(instruction.operation, Operation::WriteCpsr { .. }))
            })
    })
}

fn emit_instruction(
    s: &mut InstructionSink<'_>,
    transfers: &Transfers,
    regs: &RegionLocals,
    instruction: &Instruction,
    exit_depth: u32,
    last: bool,
    flags: u8,
) {
    let fallthrough = instruction.pc.get().wrapping_sub(regs.origin).wrapping_add(u32::from(instruction.size)) as i32;
    let writes_pc = instruction.operation.writes_pc();
    let conditional = instruction.condition != Condition::Always;
    if conditional {
        condition(s, &instruction.condition);
        s.if_(BlockType::Empty);
    }
    // Publish the faulting PC before a guest access. Other instructions need a PC only at a block boundary.
    if instruction.operation.accesses_memory() {
        s.i32_const(instruction.pc.get().wrapping_sub(regs.origin) as i32).local_set(PC);
    }
    operation(s, transfers, regs, instruction, exit_depth + u32::from(conditional), flags);
    if flags != 0 && regs.publish_mask == 0 {
        packed_cpsr(s, regs.flags_written);
        s.local_set(CPSR);
    }
    if conditional {
        if writes_pc {
            s.else_().i32_const(fallthrough).local_set(PC);
        }
        s.end();
    }
    if !writes_pc && last {
        s.i32_const(fallthrough).local_set(PC);
    }
}

fn register_value(s: &mut InstructionSink<'_>, regs: &RegionLocals, register: &Reg, pc: u32, thumb: bool) {
    if *register == Reg::PC {
        s.i32_const(pc.wrapping_add(if thumb { 4 } else { 8 }) as i32);
    } else if let Some(local) = regs.locals[usize::from(register.index())] {
        s.local_get(local);
    } else {
        s.local_get(0).i32_load(field(u64::from(register.index()) * 4));
    }
}

fn value(s: &mut InstructionSink<'_>, regs: &RegionLocals, value: &Value, pc: u32, thumb: bool) {
    match value {
        Value::Immediate(value) => {
            s.i32_const(*value as i32);
        }
        Value::Register(reg) => register_value(s, regs, reg, pc, thumb),
    }
}

fn operand(s: &mut InstructionSink<'_>, regs: &RegionLocals, operand: &Operand, pc: u32, thumb: bool, set_carry: bool) {
    if let Some((result, carry)) = operand.constant_value() {
        if set_carry && let Some(carry) = carry {
            s.i32_const(carry as i32).local_set(CARRY);
        }
        s.i32_const(result as i32);
        return;
    }
    value(s, regs, &operand.value, pc, thumb);
    if operand.amount == ShiftAmount::Immediate(0) && operand.shift != Shift::Rrx {
        return;
    }
    if !set_carry {
        match (&operand.shift, &operand.amount) {
            (Shift::Rrx, _) => {
                s.i32_const(1).i32_shr_u();
                s.local_get(CARRY);
                s.i32_const(31).i32_shl().i32_or();
            }
            (_, ShiftAmount::Immediate(0)) => {}
            (Shift::Lsl | Shift::Lsr, ShiftAmount::Immediate(count)) if *count >= 32 => {
                s.drop().i32_const(0);
            }
            (Shift::Lsl, ShiftAmount::Immediate(count)) => {
                s.i32_const(i32::from(*count)).i32_shl();
            }
            (Shift::Lsr, ShiftAmount::Immediate(count)) => {
                s.i32_const(i32::from(*count)).i32_shr_u();
            }
            (Shift::Asr, ShiftAmount::Immediate(count)) => {
                s.i32_const(i32::from((*count).min(31))).i32_shr_s();
            }
            (Shift::Ror, ShiftAmount::Immediate(count)) => {
                s.i32_const(i32::from(*count)).i32_rotr();
            }
            (shift, ShiftAmount::Register(reg)) => {
                register_value(s, regs, reg, pc, thumb);
                if *shift == Shift::Ror {
                    // Both rotate instructions already reduce the count modulo 32.
                    s.i32_rotr();
                } else {
                    s.i32_const(255).i32_and().local_tee(AMOUNT);
                    if *shift == Shift::Asr {
                        s.i32_const(31).local_get(AMOUNT).i32_const(32).i32_lt_u().select().i32_shr_s();
                    } else {
                        if *shift == Shift::Lsl {
                            s.i32_shl();
                        } else {
                            s.i32_shr_u();
                        }
                        s.i32_const(0).local_get(AMOUNT).i32_const(32).i32_lt_u().select();
                    }
                }
            }
        }
        return;
    }
    s.local_set(RIGHT);
    if operand.shift == Shift::Rrx {
        s.local_get(RIGHT).i32_const(1).i32_shr_u();
        s.local_get(CARRY);
        s.i32_const(31).i32_shl().i32_or();
        s.local_get(RIGHT).i32_const(1).i32_and().local_set(CARRY);
        return;
    }
    let reg = match operand.amount {
        ShiftAmount::Immediate(amount) => {
            let amount = i32::from(amount);
            match operand.shift {
                Shift::Lsl | Shift::Lsr => {
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
                    s.local_get(RIGHT)
                        .i32_const((amount - 1).min(31))
                        .i32_shr_u()
                        .i32_const(1)
                        .i32_and()
                        .local_set(CARRY);
                    s.local_get(RIGHT).i32_const(amount.min(31)).i32_shr_s();
                }
                Shift::Ror => {
                    s.local_get(RIGHT).i32_const(amount).i32_rotr();
                    s.local_tee(RIGHT).local_get(RIGHT).i32_const(31).i32_shr_u().local_set(CARRY);
                }
                Shift::Rrx => {}
            }
            return;
        }
        ShiftAmount::Register(ref reg) => reg,
    };
    register_value(s, regs, reg, pc, thumb);
    s.i32_const(255).i32_and();
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
            s.else_().i32_const(0).local_set(RIGHT);
            s.i32_const(0).local_set(CARRY);
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
            s.local_get(RIGHT).local_get(AMOUNT).i32_rotr().local_set(RIGHT);
            s.local_get(RIGHT).i32_const(31).i32_shr_u().local_set(CARRY);
        }
        Shift::Rrx => {}
    }
    s.end().local_get(RIGHT);
}

fn unpack_flags(s: &mut InstructionSink<'_>, mask: u32) {
    if mask & N != 0 {
        s.local_get(CPSR).local_set(NEGATIVE);
    }
    if mask & Z != 0 {
        s.local_get(CPSR).i32_const(30).i32_shr_u().i32_const(1).i32_and().local_set(ZERO);
    }
    if mask & C != 0 {
        s.local_get(CPSR).i32_const(29).i32_shr_u().i32_const(1).i32_and().local_set(CARRY);
    }
    if mask & V != 0 {
        s.local_get(CPSR).i32_const(3).i32_shl().local_set(OVERFLOW);
    }
}

// Keep explicit MRS expressions visible to the Wasm optimizer. Host publication shares the same merge through a module function.
fn merge_flags(s: &mut InstructionSink<'_>, mask: u32, cpsr: u32, [n, z, c, v]: [u32; 4]) {
    s.local_get(cpsr);
    if mask != 0 {
        s.i32_const(!mask as i32)
            .i32_and()
            .local_get(n)
            .i32_const(i32::MIN)
            .i32_and()
            .i32_or()
            .local_get(z)
            .i32_const(30)
            .i32_shl()
            .i32_or();
        if mask & C != 0 {
            s.local_get(c).i32_const(29).i32_shl().i32_or();
        }
        if mask & V != 0 {
            s.local_get(v).i32_const(31).i32_shr_u().i32_const(28).i32_shl().i32_or();
        }
    }
}

fn packed_cpsr(s: &mut InstructionSink<'_>, mask: u32) {
    s.local_get(CPSR);
    if mask != 0 {
        s.local_get(NEGATIVE).local_get(ZERO);
        if mask & C != 0 {
            s.local_get(CARRY);
        }
        if mask & V != 0 {
            s.local_get(OVERFLOW);
        }
        s.call(if mask & V != 0 {
            PACK_NZCV
        } else if mask & C != 0 {
            PACK_NZC
        } else {
            PACK_NZ
        });
    }
}

fn condition(s: &mut InstructionSink<'_>, condition: &Condition) {
    match condition {
        Condition::Eq | Condition::Ne => {
            s.local_get(ZERO);
        }
        Condition::Cs | Condition::Cc => {
            s.local_get(CARRY);
        }
        Condition::Mi | Condition::Pl => {
            s.local_get(NEGATIVE).i32_const(0).i32_lt_s();
        }
        Condition::Vs | Condition::Vc => {
            s.local_get(OVERFLOW).i32_const(0).i32_lt_s();
        }
        Condition::Hi | Condition::Ls => {
            s.local_get(CARRY).local_get(ZERO).i32_eqz().i32_and();
        }
        Condition::Ge | Condition::Lt | Condition::Gt | Condition::Le => {
            s.local_get(NEGATIVE).local_get(OVERFLOW).i32_xor().i32_const(0).i32_ge_s();
            if matches!(condition, Condition::Gt | Condition::Le) {
                s.local_get(ZERO).i32_eqz().i32_and();
            }
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
    if exchange {
        s.local_set(PC);
        s.local_get(0).local_get(CPSR).i32_const(!0x20).i32_and();
        s.local_get(PC)
            .i32_const(1)
            .i32_and()
            .i32_const(5)
            .i32_shl()
            .i32_or()
            .local_tee(CPSR)
            .i32_store(field(64));
        s.local_get(PC).i32_const(!1).i32_const(!3).local_get(PC).i32_const(1).i32_and().select();
    } else {
        s.i32_const(if thumb { !1 } else { !3 });
    }
    s.i32_and().local_get(ORIGIN).i32_sub().local_set(PC);
}

fn multiply_flags(s: &mut InstructionSink<'_>, wide: bool) {
    if wide {
        s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64().local_set(NEGATIVE);
        s.local_get(WIDE).i64_eqz().local_set(ZERO);
    } else {
        s.local_get(RESULT).local_tee(NEGATIVE).i32_eqz().local_set(ZERO);
    }
}

// Capture both the effective address and final writeback before any destination changes.
fn memory_address(s: &mut InstructionSink<'_>, regs: &RegionLocals, address: &MemoryOperand, pc: u32, thumb: bool) {
    value(s, regs, &address.base, pc, thumb);
    if !address.pre_index && address.write_back.is_none() {
        return;
    }
    let offset = address.offset.constant_value().map(|(value, _)| value);
    if offset == Some(0) {
        if address.write_back.is_some() {
            s.local_tee(RESULT);
        }
        return;
    }
    if !address.pre_index {
        s.local_tee(LEFT);
    }
    if let Some(offset) = offset {
        s.i32_const(if address.subtract { offset.wrapping_neg() } else { offset } as i32)
            .i32_add();
    } else {
        operand(s, regs, &address.offset, pc, thumb, false);
        if address.subtract {
            s.i32_sub();
        } else {
            s.i32_add();
        }
    }
    if !address.pre_index {
        s.local_set(RESULT).local_get(LEFT);
    } else if address.write_back.is_some() {
        s.local_tee(RESULT);
    }
}

fn scalar_base(address: &MemoryOperand) -> Option<usize> {
    let Value::Register(reg) = &address.base else { return None };
    (!address.pre_index || address.offset.constant_value().is_some_and(|(offset, _)| offset == 0)).then(|| usize::from(reg.index()))
}

fn load_page(s: &mut InstructionSink<'_>, address: u32) {
    // Each directory entry is a four-byte nullable page pointer.
    s.local_get(PAGE_TABLE)
        .local_get(address)
        .i32_const(16)
        .i32_shr_u()
        .i32_const(2)
        .i32_shl()
        .i32_add()
        .i32_load(field(0));
}

fn scalar_address(s: &mut InstructionSink<'_>, width: &Width, exit_depth: u32, address: Option<u32>, translated: Option<u32>) -> u64 {
    let alignment_mask = match width {
        Width::Byte => 0,
        Width::Half => 1,
        Width::Word => 3,
    };
    if let Some(address) = address {
        if address & alignment_mask != 0 {
            s.br(exit_depth);
            return 0;
        }
    } else if alignment_mask != 0 {
        s.local_tee(LEFT).i32_const(alignment_mask as i32).i32_and().br_if(exit_depth);
    } else {
        s.local_set(LEFT);
    }
    s.local_get(PAGE_TABLE).i32_eqz().br_if(exit_depth - 3);
    if let Some(local) = translated {
        s.local_get(local).i32_eqz().br_if(exit_depth);
        s.local_get(local);
        return 0;
    }
    if let Some(address) = address {
        s.local_get(PAGE_TABLE).i32_load(field(u64::from(address >> 16) * 4));
    } else {
        load_page(s, LEFT);
    }
    s.local_tee(PAGE_POINTER).i32_eqz().br_if(exit_depth);
    s.local_get(PAGE_POINTER);
    if let Some(address) = address {
        u64::from(address & 65535)
    } else {
        s.local_get(LEFT);
        s.global_get(0);
        s.i32_and().i32_add();
        0
    }
}

// Repeated register lists use one unrolled transfer with statically assigned frame slots.
fn compile_transfer(registers: u16, load: bool) -> Function {
    const FRAME: u32 = 0;
    const ACCESS: u32 = 1;
    const GUEST_PC: u32 = 2;
    const STATUS: u32 = 3;
    const ADDRESS: u32 = 4;
    const SPANS: u32 = 5;
    const FIRST: u32 = 6;
    const SECOND_BIAS: u32 = 7;
    const FIRST_LEN: u32 = 8;

    let words = registers.count_ones();
    let mut function = Function::new([(1, ValType::I64), (3, ValType::I32)]);
    let mut s = function.instructions();
    s.local_get(FRAME).local_get(GUEST_PC).i32_store(field(60));
    s.local_get(FRAME).local_get(STATUS).i32_store(field(64));
    s.local_get(ACCESS)
        .local_get(ADDRESS)
        .i32_const(words as i32)
        .call(WORD_RANGE)
        .local_tee(SPANS)
        .i64_eqz()
        .if_(BlockType::Empty);
    s.i32_const(0).return_().end();
    s.local_get(SPANS).i32_wrap_i64().local_set(FIRST);
    if words > 1 {
        s.local_get(FRAME).i32_load(field(76)).local_set(FIRST_LEN);
        s.local_get(SPANS)
            .i64_const(32)
            .i64_shr_u()
            .i32_wrap_i64()
            .local_get(FIRST_LEN)
            .i32_sub()
            .local_set(SECOND_BIAS);
    }

    // Consecutive ordinary registers can move in pairs when the admission boundary
    // also falls between pairs. Other lists use the same unrolled word accesses.
    let consecutive = registers & 0x8000 == 0 && words >= 4 && (registers >> registers.trailing_zeros()).trailing_ones() == words;
    for paired in [true, false] {
        if paired {
            if !consecutive {
                continue;
            }
            s.local_get(FIRST_LEN).i32_const(4).i32_and().i32_eqz().if_(BlockType::Empty);
        }
        let mut offset = 0;
        let mut list = registers;
        while list != 0 {
            let reg = list.trailing_zeros();
            list &= list - 1;
            let wide = paired && list != 0;
            if wide {
                list &= list - 1;
            }
            if load && reg != 15 {
                s.local_get(FRAME);
            }
            s.local_get(FIRST);
            if offset != 0 {
                // The second-span bias can wrap. Perform the addition at guest pointer width.
                s.local_get(SECOND_BIAS)
                    .local_get(FIRST_LEN)
                    .i32_const(offset)
                    .i32_gt_u()
                    .select()
                    .i32_const(offset)
                    .i32_add();
            }
            let slot = field(u64::from(reg) * 4);
            if load {
                if reg == 15 {
                    // A zero PC is a valid target. Return its non-null word pointer to signal success.
                    s.return_();
                } else if wide {
                    s.i64_load(field(0)).i64_store(slot);
                } else {
                    s.i32_load(field(0)).i32_store(slot);
                }
            } else if reg == 15 {
                s.local_get(GUEST_PC).i32_const(12).i32_add().i32_store(field(0));
            } else {
                s.local_get(FRAME);
                if wide {
                    s.i64_load(slot).i64_store(field(0));
                } else {
                    s.i32_load(slot).i32_store(field(0));
                }
            }
            offset += if wide { 8 } else { 4 };
        }
        if paired {
            s.i32_const(1).return_().end();
        }
    }
    s.i32_const(1).end();
    function
}

fn transfer_words(s: &mut InstructionSink<'_>, transfers: &Transfers, regs: &RegionLocals, registers: u16, load: bool, exit_depth: u32) {
    regs.publish(s);
    if regs.retry_pages {
        s.i32_const(0).local_set(PAGE_TABLE);
    }
    s.local_get(0).local_get(1).local_get(PC).local_get(ORIGIN).i32_add();
    packed_cpsr(s, regs.publish_mask);
    s.local_get(LEFT).call(transfers[&(registers, load)]);
    if load && registers & 0x8000 != 0 {
        s.local_tee(RANGE_FIRST);
    }
    s.i32_eqz().br_if(exit_depth);
    if load {
        for (index, local) in regs.locals.iter().enumerate() {
            if let Some(local) = local
                && registers & (1 << index) != 0
            {
                s.local_get(0).i32_load(field(index as u64 * 4)).local_set(*local);
            }
        }
        if registers & 0x8000 != 0 {
            s.local_get(RANGE_FIRST).i32_load(field(0));
            commit_pc(s, regs.thumb, true);
        }
    }
}

fn operation(s: &mut InstructionSink<'_>, transfers: &Transfers, regs: &RegionLocals, instruction: &Instruction, exit_depth: u32, flags: u8) {
    match instruction.operation {
        Operation::Alu {
            ref op,
            ref destination,
            ref left,
            ref right,
            set_flags,
        } => {
            let destination = destination.as_ref();
            let set_flags = set_flags && flags != 0;
            if !set_flags && destination.is_none() {
                return;
            }
            if (!set_flags || matches!(op, AluOp::Move | AluOp::Not))
                && let Some((right, carry)) = right.constant_value()
                && let Some(result) = op.constant_result(left, right)
            {
                if let Some(destination) = destination {
                    if *destination == Reg::PC {
                        s.i32_const(result as i32);
                        commit_pc(s, regs.thumb, false);
                    } else {
                        regs.start_store(s, destination);
                        s.i32_const(result as i32);
                        regs.finish_store(s, destination);
                    }
                }
                if set_flags {
                    if flags & 8 != 0 {
                        s.i32_const(if result & N == 0 { 0 } else { -1 }).local_set(NEGATIVE);
                    }
                    if flags & 4 != 0 {
                        s.i32_const(i32::from(result == 0)).local_set(ZERO);
                    }
                    if flags & 2 != 0
                        && let Some(carry) = carry
                    {
                        s.i32_const(carry as i32).local_set(CARRY);
                    }
                }
                return;
            }
            let arithmetic = matches!(
                op,
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry
            );
            let immediate = right.constant_value().map(|(value, _)| value);
            let zero_arithmetic = matches!(op, AluOp::Add | AluOp::Sub) && immediate == Some(0);
            let preserves_carry = right.amount == ShiftAmount::Immediate(0) && right.shift != Shift::Rrx;
            let reverse = matches!(op, AluOp::ReverseSub | AluOp::ReverseSubCarry);
            if let Some(reg) = destination
                && *reg != Reg::PC
            {
                regs.start_store(s, reg);
            }
            if !reverse && !matches!(op, AluOp::Move | AluOp::Not | AluOp::CountLeadingZeros) {
                value(s, regs, left, instruction.pc.get(), regs.thumb);
            }
            let flag_immediate = immediate.filter(|_| set_flags && matches!(op, AluOp::Add | AluOp::Sub));
            if !zero_arithmetic && flag_immediate.is_none() {
                operand(
                    s,
                    regs,
                    right,
                    instruction.pc.get(),
                    regs.thumb,
                    set_flags && flags & 2 != 0 && !arithmetic && *op != AluOp::Multiply && !preserves_carry,
                );
            }
            if reverse {
                value(s, regs, left, instruction.pc.get(), regs.thumb);
            }
            match op {
                AluOp::Add | AluOp::Sub if zero_arithmetic => {}
                AluOp::Add | AluOp::Sub if let Some(value) = flag_immediate => {
                    s.local_tee(LEFT)
                        .i32_const(if *op == AluOp::Add { value } else { value.wrapping_neg() } as i32)
                        .i32_add();
                }
                AluOp::Add | AluOp::AddCarry | AluOp::Sub | AluOp::SubCarry | AluOp::ReverseSub | AluOp::ReverseSubCarry => {
                    let subtract = !matches!(op, AluOp::Add | AluOp::AddCarry);
                    if !set_flags || flags & 3 == 0 {
                        if subtract {
                            s.i32_sub();
                        } else {
                            s.i32_add();
                        }
                        if matches!(op, AluOp::AddCarry | AluOp::SubCarry | AluOp::ReverseSubCarry) {
                            s.local_get(CARRY);
                            s.i32_add();
                            if subtract {
                                s.i32_const(1).i32_sub();
                            }
                        }
                    } else if matches!(op, AluOp::Add | AluOp::Sub | AluOp::ReverseSub) {
                        s.local_set(RIGHT).local_tee(LEFT).local_get(RIGHT);
                        if subtract {
                            s.i32_sub();
                        } else {
                            s.i32_add();
                        }
                    } else {
                        s.local_set(RIGHT).local_set(LEFT);
                        // A + ~B + C implements subtraction with no-borrow carry semantics.
                        s.local_get(LEFT).i64_extend_i32_u().local_get(RIGHT);
                        if subtract {
                            s.i32_const(-1).i32_xor();
                        }
                        s.i64_extend_i32_u().i64_add();
                        s.local_get(CARRY);
                        s.i64_extend_i32_u().i64_add().local_tee(WIDE).i32_wrap_i64();
                    }
                }
                AluOp::And => {
                    s.i32_and();
                }
                AluOp::Or => {
                    s.i32_or();
                }
                AluOp::Xor => {
                    s.i32_xor();
                }
                AluOp::Move => {}
                AluOp::BitClear => {
                    s.i32_const(-1).i32_xor().i32_and();
                }
                AluOp::Not => {
                    s.i32_const(-1).i32_xor();
                }
                AluOp::Multiply => {
                    s.i32_mul();
                }
                AluOp::CountLeadingZeros => {
                    s.i32_clz();
                }
            }
            let result_local = if set_flags && flags & 8 != 0 { NEGATIVE } else { RESULT };
            if let Some(destination) = destination {
                if set_flags {
                    s.local_tee(result_local);
                }
                if *destination == Reg::PC {
                    commit_pc(s, regs.thumb, false);
                } else {
                    regs.finish_store(s, destination);
                }
            } else if set_flags {
                s.local_set(result_local);
            } else {
                s.drop();
            }
            if set_flags {
                if flags & 4 != 0 {
                    s.local_get(result_local).i32_eqz().local_set(ZERO);
                }
                if arithmetic {
                    if zero_arithmetic {
                        if flags & 2 != 0 {
                            s.i32_const(i32::from(*op == AluOp::Sub)).local_set(CARRY);
                        }
                        if flags & 1 != 0 {
                            s.i32_const(0).local_set(OVERFLOW);
                        }
                    } else if let Some(v) = flag_immediate {
                        let add = *op == AluOp::Add;
                        if flags & 2 != 0 {
                            s.local_get(LEFT)
                                .i32_const(if add { v.wrapping_neg() } else { v } as i32)
                                .i32_ge_u()
                                .local_set(CARRY);
                        }
                        if flags & 1 != 0 {
                            let delta = i64::from(v as i32) * if add { 1 } else { -1 };
                            s.local_get(LEFT);
                            if delta > 0 {
                                s.i32_const((i64::from(i32::MAX) - delta) as i32).i32_gt_s();
                            } else {
                                s.i32_const((i64::from(i32::MIN) - delta) as i32).i32_lt_s();
                            }
                            s.i32_const(31).i32_shl().local_set(OVERFLOW);
                        }
                    } else {
                        if flags & 2 != 0 {
                            if *op == AluOp::Add {
                                s.local_get(result_local).local_get(LEFT).i32_lt_u();
                            } else if matches!(op, AluOp::Sub | AluOp::ReverseSub) {
                                s.local_get(LEFT).local_get(RIGHT).i32_ge_u();
                            } else {
                                s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64();
                            }
                            s.local_set(CARRY);
                        }
                        if flags & 1 != 0 {
                            s.local_get(RIGHT)
                                .local_get(if matches!(op, AluOp::Add | AluOp::AddCarry) {
                                    result_local
                                } else {
                                    LEFT
                                })
                                .i32_xor();
                            s.local_get(LEFT).local_get(result_local).i32_xor().i32_and().local_set(OVERFLOW);
                        }
                    }
                }
            }
        }
        Operation::Branch {
            ref target,
            ref link,
            exchange,
        } => {
            match target {
                BranchTarget::Address(address) => {
                    let target_thumb = if exchange { address.get() & 1 != 0 } else { regs.thumb };
                    if exchange {
                        s.local_get(0).local_get(CPSR);
                        if target_thumb {
                            s.i32_const(0x20).i32_or();
                        } else {
                            s.i32_const(!0x20).i32_and();
                        }
                        s.local_tee(CPSR).i32_store(field(64));
                    }
                    s.i32_const((address.get() & if target_thumb { !1 } else { !3 }).wrapping_sub(regs.origin) as i32)
                        .local_set(PC);
                }
                BranchTarget::Register(register) => {
                    register_value(s, regs, register, instruction.pc.get(), regs.thumb);
                    commit_pc(s, regs.thumb, exchange);
                }
            }
            if let Some(link) = link {
                regs.start_store(s, &Reg::LR);
                s.i32_const(link.get() as i32);
                regs.finish_store(s, &Reg::LR);
            }
        }
        Operation::Load { ref address, ref width, .. } | Operation::Store { ref address, ref width, .. } => {
            if let Operation::Load { ref destination, .. } = instruction.operation
                && *destination != Reg::PC
            {
                regs.start_store(s, destination);
            }
            let literal = if let Some((effective, indexed)) = address.constant_address() {
                if address.write_back.is_some() {
                    s.i32_const(indexed as i32).local_set(RESULT);
                }
                Some(effective)
            } else {
                memory_address(s, regs, address, instruction.pc.get(), regs.thumb);
                None
            };
            let translated = scalar_base(address).and_then(|index| regs.translated[index]);
            let offset = scalar_address(s, width, exit_depth, literal, translated);
            if let Operation::Store { value: ref source, .. } = instruction.operation {
                if *source == Value::Register(Reg::PC) {
                    s.i32_const(instruction.pc.get().wrapping_add(12) as i32);
                } else {
                    value(s, regs, source, instruction.pc.get(), regs.thumb);
                }
                match width {
                    Width::Byte => {
                        s.i32_store8(MemArg { align: 0, ..field(offset) });
                    }
                    Width::Half => {
                        s.i32_store16(MemArg { align: 1, ..field(offset) });
                    }
                    Width::Word => {
                        s.i32_store(field(offset));
                    }
                }
            } else if let Operation::Load { ref destination, signed, .. } = instruction.operation {
                match (width, signed) {
                    (Width::Byte, false) => {
                        s.i32_load8_u(MemArg { align: 0, ..field(offset) });
                    }
                    (Width::Byte, true) => {
                        s.i32_load8_s(MemArg { align: 0, ..field(offset) });
                    }
                    (Width::Half, false) => {
                        s.i32_load16_u(MemArg { align: 1, ..field(offset) });
                    }
                    (Width::Half, true) => {
                        s.i32_load16_s(MemArg { align: 1, ..field(offset) });
                    }
                    (Width::Word, _) => {
                        s.i32_load(field(offset));
                    }
                }
                if *destination == Reg::PC {
                    commit_pc(s, regs.thumb, true);
                } else {
                    regs.finish_store(s, destination);
                }
            }
            if let Some(register) = &address.write_back {
                regs.store_local(s, register, RESULT);
            }
        }
        Operation::MultiplyAccumulate {
            ref destination,
            ref left,
            ref right,
            ref accumulate,
            set_flags,
        } => {
            register_value(s, regs, left, instruction.pc.get(), regs.thumb);
            register_value(s, regs, right, instruction.pc.get(), regs.thumb);
            s.i32_mul();
            register_value(s, regs, accumulate, instruction.pc.get(), regs.thumb);
            s.i32_add().local_set(RESULT);
            regs.store_local(s, destination, RESULT);
            if set_flags && flags != 0 {
                multiply_flags(s, false);
            }
        }
        Operation::MultiplyLong {
            ref low,
            ref high,
            ref left,
            ref right,
            signed,
            accumulate,
            set_flags,
        } => {
            for register in [left, right] {
                register_value(s, regs, register, instruction.pc.get(), regs.thumb);
                if signed {
                    s.i64_extend_i32_s();
                } else {
                    s.i64_extend_i32_u();
                }
            }
            s.i64_mul();
            if accumulate {
                register_value(s, regs, low, instruction.pc.get(), regs.thumb);
                s.i64_extend_i32_u();
                register_value(s, regs, high, instruction.pc.get(), regs.thumb);
                s.i64_extend_i32_u().i64_const(32).i64_shl().i64_or().i64_add();
            }
            s.local_set(WIDE);
            regs.start_store(s, low);
            s.local_get(WIDE).i32_wrap_i64();
            regs.finish_store(s, low);
            regs.start_store(s, high);
            s.local_get(WIDE).i64_const(32).i64_shr_u().i32_wrap_i64();
            regs.finish_store(s, high);
            if set_flags && flags != 0 {
                multiply_flags(s, true);
            }
        }
        Operation::ReadCpsr { ref destination } => {
            regs.start_store(s, destination);
            merge_flags(s, regs.publish_mask, CPSR, [NEGATIVE, ZERO, CARRY, OVERFLOW]);
            regs.finish_store(s, destination);
        }
        Operation::WriteCpsr { value: ref source, mask } => {
            s.local_get(0).local_get(CPSR).i32_const(!mask as i32).i32_and();
            value(s, regs, source, instruction.pc.get(), regs.thumb);
            s.i32_const(mask as i32).i32_and().i32_or().local_tee(CPSR).i32_store(field(64));
            unpack_flags(s, mask);
        }
        Operation::MultipleTransfer {
            ref base,
            registers,
            increment,
            before,
            write_back,
            load,
        } => {
            let bytes = registers.count_ones() as i32 * 4;
            let change = if increment { bytes } else { -bytes };
            let start = if increment {
                if before { 4 } else { 0 }
            } else {
                -bytes + if before { 0 } else { 4 }
            };
            register_value(s, regs, base, instruction.pc.get(), regs.thumb);
            if write_back && start != change {
                s.local_tee(LEFT).i32_const(change).i32_add().local_set(RESULT).local_get(LEFT);
            }
            if start != 0 {
                s.i32_const(start).i32_add();
            }
            if write_back && start == change {
                s.local_tee(RESULT);
            }
            s.local_set(LEFT);
            transfer_words(s, transfers, regs, registers, load, exit_depth);
            if write_back {
                regs.store_local(s, base, RESULT);
            }
        }
        Operation::DoubleTransfer {
            ref register,
            ref address,
            load,
        } => {
            memory_address(s, regs, address, instruction.pc.get(), regs.thumb);
            s.local_set(LEFT);
            transfer_words(s, transfers, regs, 3 << register.index(), load, exit_depth);
            if let Some(base) = &address.write_back {
                regs.store_local(s, base, RESULT);
            }
        }
        Operation::Swap {
            ref destination,
            ref address,
            value: ref source,
            ref width,
        } => {
            register_value(s, regs, address, instruction.pc.get(), regs.thumb);
            register_value(s, regs, source, instruction.pc.get(), regs.thumb);
            s.local_set(RIGHT);
            scalar_address(s, width, exit_depth, None, None);
            s.local_set(RANGE_FIRST);
            if matches!(width, Width::Word) {
                s.local_get(RANGE_FIRST).i32_load(field(0)).local_set(RESULT);
                s.local_get(RANGE_FIRST).local_get(RIGHT).i32_store(field(0));
            } else {
                s.local_get(RANGE_FIRST).i32_load8_u(MemArg { align: 0, ..field(0) }).local_set(RESULT);
                s.local_get(RANGE_FIRST).local_get(RIGHT).i32_store8(MemArg { align: 0, ..field(0) });
            }
            regs.store_local(s, destination, RESULT);
        }
        Operation::Nop => {}
    }
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec};
    use wie_core_arm_aot::ir::MemoryAddress;

    use wie_core_arm_aot::{CompileRegion, CompileRequest, RegionKey, ir::BasicBlock};

    use crate::Compiler;

    use super::*;

    #[test]
    fn conditions_emit_direct_bit_comparisons() {
        for (positive, negative, bytes) in [
            (Condition::Eq, Condition::Ne, vec![0x20, ZERO as u8]),
            (Condition::Cs, Condition::Cc, vec![0x20, CARRY as u8]),
            (Condition::Mi, Condition::Pl, vec![0x20, NEGATIVE as u8, 0x41, 0, 0x48]),
            (Condition::Vs, Condition::Vc, vec![0x20, OVERFLOW as u8, 0x41, 0, 0x48]),
            (Condition::Hi, Condition::Ls, vec![0x20, CARRY as u8, 0x20, ZERO as u8, 0x45, 0x71]),
            (
                Condition::Ge,
                Condition::Lt,
                vec![0x20, NEGATIVE as u8, 0x20, OVERFLOW as u8, 0x73, 0x41, 0, 0x4e],
            ),
            (
                Condition::Gt,
                Condition::Le,
                vec![
                    0x20,
                    NEGATIVE as u8,
                    0x20,
                    OVERFLOW as u8,
                    0x73,
                    0x41,
                    0,
                    0x4e,
                    0x20,
                    ZERO as u8,
                    0x45,
                    0x71,
                ],
            ),
        ] {
            for code in [&positive, &negative] {
                let mut function = Function::new([]);
                condition(&mut function.instructions(), code);
                let mut expected = vec![0];
                expected.extend_from_slice(&bytes);
                if *code == negative {
                    expected.push(0x45);
                }
                assert_eq!(function.into_raw_body(), expected, "condition {:?}", core::mem::discriminant(code));
            }
        }
    }

    #[test]
    fn large_batches_compile_into_a_single_module() {
        let request = CompileRequest {
            images: Arc::from([]),
            max_region_instructions: 512,
            max_region_blocks: 128,
            regions: Box::new((0..48).map(|step| {
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
                                    pc: MemoryAddress::new(pc + index * 4),
                                    size: 4,
                                    condition: Condition::Le,
                                    operation: Operation::Alu {
                                        op: AluOp::ReverseSubCarry,
                                        destination: Some(Reg::new(0)),
                                        left: Value::Register(Reg::new(0)),
                                        right: Operand {
                                            value: Value::Register(Reg::new(1)),
                                            shift: Shift::Lsl,
                                            amount: ShiftAmount::Register(Reg::new(2)),
                                        },
                                        set_flags: true,
                                    },
                                })
                                .collect(),
                        }],
                    },
                })
            })),
        };
        let mut compiler = Compiler::new(request);
        assert!(!compiler.step().unwrap());
        assert!(compiler.artifact.manifest.is_empty());
        assert!(!compiler.step().unwrap());
        assert!(compiler.artifact.manifest.is_empty());
        while !compiler.step().unwrap() {}
        let artifact = compiler.finish();
        assert_eq!(artifact.manifest.iter().filter(|region| !region.entry.thumb).count(), 24);
        assert_eq!(
            artifact.manifest.iter().map(|region| region.instruction_pcs.len()).sum::<usize>(),
            24 * 512
        );
        assert!(artifact.chunks.iter().map(Vec::len).sum::<usize>() > 512 * 1024);
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
            let shift_id = core::mem::discriminant(&shift);
            let mut actual = Function::new([]);
            operand(
                &mut actual.instructions(),
                &RegionLocals::default(),
                &Operand {
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
            if let Some(carry) = carry {
                s.i32_const(carry).local_set(CARRY);
            }
            s.i32_const(result as i32);
            assert_eq!(
                actual.into_raw_body(),
                expected.into_raw_body(),
                "shift {shift_id:?}: {input:#x} by {amount}"
            );
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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
            let op_id = core::mem::discriminant(&op);
            let set_flags = matches!(op, AluOp::Move | AluOp::Not);
            let move_op = op == AluOp::Move;
            let mut actual = Function::new([]);
            operation(
                &mut actual.instructions(),
                &Transfers::new(),
                &RegionLocals::default(),
                &Instruction {
                    pc: MemoryAddress::new(0x1000),
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Alu {
                        op,
                        destination: Some(Reg::new(0)),
                        left: Value::Immediate(left),
                        right: Operand {
                            value: Value::Immediate(input),
                            shift,
                            amount: ShiftAmount::Immediate(amount),
                        },
                        set_flags,
                    },
                },
                0,
                0xf,
            );
            let mut expected = Function::new([]);
            let mut s = expected.instructions();
            s.local_get(0).i32_const(result as i32).i32_store(field(0));
            if set_flags {
                s.i32_const(if result & N == 0 { 0 } else { -1 }).local_set(NEGATIVE);
                s.i32_const(i32::from(result == 0)).local_set(ZERO);
                if move_op {
                    s.i32_const(1).local_set(CARRY);
                }
            }
            assert_eq!(
                actual.into_raw_body(),
                expected.into_raw_body(),
                "operation {op_id:?}: {left:#x}, {input:#x}"
            );
        }
    }
}
