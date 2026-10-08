use std::mem::{offset_of, size_of};

use cranelift_codegen::ir::{Endianness, InstBuilder, MemFlagsData, Value, condcodes::IntCC, types};

use wie_arm_jit_types::{
    CompiledExit, MemoryPage,
    ir::{self, Instruction, MemoryOperand, Operation, Reg, Width},
};

use super::Emitter;

impl Emitter<'_> {
    pub(super) fn emit_memory(&mut self, instruction: &Instruction) -> Option<Value> {
        match &instruction.operation {
            Operation::Load {
                destination,
                address,
                width,
                signed,
            } => {
                let (effective, writeback) = self.memory_address(address);
                self.check_alignment(effective, width);
                let pointer = self.guest_pointer(effective);
                let value = self.load_memory(pointer, width, *signed);
                let next = if *destination == Reg::PC {
                    Some(self.pc_write(value, true))
                } else {
                    self.store_register(destination, value);
                    None
                };
                if let Some(register) = &address.write_back {
                    self.store_register(register, writeback);
                }
                next
            }
            Operation::Store { value, address, width } => {
                let (effective, writeback) = self.memory_address(address);
                self.check_alignment(effective, width);
                let pointer = self.guest_pointer(effective);
                let value = if *value == ir::Value::Register(Reg::PC) {
                    self.builder
                        .ins()
                        .iconst(types::I32, i64::from(self.pc.wrapping_add(if self.thumb { 4 } else { 12 })))
                } else {
                    self.value(value)
                };
                self.store_memory(pointer, width, value);
                if let Some(register) = &address.write_back {
                    self.store_register(register, writeback);
                }
                None
            }
            Operation::Swap {
                destination,
                address,
                value,
                width,
            } => {
                let address = self.register(address);
                let source = self.register(value);
                self.check_alignment(address, width);
                let pointer = self.guest_pointer(address);
                let loaded = self.load_memory(pointer, width, false);
                self.store_memory(pointer, width, source);
                self.store_register(destination, loaded);
                None
            }
            Operation::MultipleTransfer {
                base,
                registers,
                increment,
                before,
                write_back,
                load,
            } => {
                let bytes = i64::from(registers.count_ones()) * 4;
                let base_value = self.register(base);
                let writeback = self.builder.ins().iadd_imm_s(base_value, if *increment { bytes } else { -bytes });
                let offset = if *increment {
                    if *before { 4 } else { 0 }
                } else {
                    -bytes + if *before { 0 } else { 4 }
                };
                let address = self.builder.ins().iadd_imm_s(base_value, offset);
                let next = self.transfer_words(address, *registers, *load);
                if *write_back {
                    self.store_register(base, writeback);
                }
                next
            }
            Operation::DoubleTransfer { register, address, load } => {
                let (effective, writeback) = self.memory_address(address);
                // Word-range admission checks bits 0..1; ARMv5 double transfers also require bit 2 clear.
                let bit = self.builder.ins().band_imm_s(effective, 4);
                let unaligned = self.builder.ins().icmp_imm_s(IntCC::NotEqual, bit, 0);
                self.guard(unaligned, CompiledExit::InterpretOne);
                let next = self.transfer_words(effective, 3 << register.index(), *load);
                if let Some(register) = &address.write_back {
                    self.store_register(register, writeback);
                }
                next
            }
            Operation::Alu { .. }
            | Operation::Branch { .. }
            | Operation::MultiplyAccumulate { .. }
            | Operation::MultiplyLong { .. }
            | Operation::ReadCpsr { .. }
            | Operation::WriteCpsr { .. }
            | Operation::Nop => None,
        }
    }

    fn memory_address(&mut self, address: &MemoryOperand) -> (Value, Value) {
        let base = self.value(&address.base);
        let (offset, _) = self.operand(&address.offset);
        let writeback = if address.subtract {
            self.builder.ins().isub(base, offset)
        } else {
            self.builder.ins().iadd(base, offset)
        };
        (if address.pre_index { writeback } else { base }, writeback)
    }

    fn check_alignment(&mut self, address: Value, width: &Width) {
        let mask = match width {
            Width::Byte => return,
            Width::Half => 1,
            Width::Word => 3,
        };
        let low = self.builder.ins().band_imm_s(address, mask);
        let unaligned = self.builder.ins().icmp_imm_s(IntCC::NotEqual, low, 0);
        self.guard(unaligned, CompiledExit::InterpretOne);
    }

    fn guest_pointer(&mut self, address: Value) -> Value {
        let page = self.builder.ins().ushr_imm_s(address, 16);
        let page = self.builder.ins().uextend(self.ptr_type, page);
        let offset = self.builder.ins().imul_imm_s(page, size_of::<MemoryPage>() as i64);
        let entry = self.builder.ins().iadd(self.pages, offset);
        let pointer = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlagsData::trusted(), entry, offset_of!(MemoryPage, bytes) as i32);
        let unmapped = self.builder.ins().icmp_imm_s(IntCC::Equal, pointer, 0);
        self.guard(unmapped, CompiledExit::InterpretOne);
        let offset = self.builder.ins().band_imm_s(address, 0xffff);
        let offset = self.builder.ins().uextend(self.ptr_type, offset);
        self.builder.ins().iadd(pointer, offset)
    }

    fn load_memory(&mut self, pointer: Value, width: &Width, signed: bool) -> Value {
        let flags = MemFlagsData::new().with_endianness(Endianness::Little);
        match (width, signed) {
            (Width::Byte, false) => self.builder.ins().uload8(types::I32, flags, pointer, 0),
            (Width::Byte, true) => self.builder.ins().sload8(types::I32, flags, pointer, 0),
            (Width::Half, false) => self.builder.ins().uload16(types::I32, flags, pointer, 0),
            (Width::Half, true) => self.builder.ins().sload16(types::I32, flags, pointer, 0),
            (Width::Word, _) => self.builder.ins().load(types::I32, flags, pointer, 0),
        }
    }

    fn store_memory(&mut self, pointer: Value, width: &Width, value: Value) {
        let flags = MemFlagsData::new().with_endianness(Endianness::Little);
        match width {
            Width::Byte => {
                self.builder.ins().istore8(flags, value, pointer, 0);
            }
            Width::Half => {
                self.builder.ins().istore16(flags, value, pointer, 0);
            }
            Width::Word => {
                self.builder.ins().store(flags, value, pointer, 0);
            }
        }
    }

    fn transfer_words(&mut self, address: Value, registers: u16, load: bool) -> Option<Value> {
        self.check_alignment(address, &Width::Word);
        // Admit the entire range before data access, including a wrapped guest page.
        let transfers: Vec<_> = (0..16)
            .filter(|register| registers & (1 << register) != 0)
            .enumerate()
            .map(|(index, register)| {
                let address = self.builder.ins().iadd_imm_s(address, index as i64 * 4);
                (Reg::new(register), self.guest_pointer(address))
            })
            .collect();
        let mut next = None;
        for (register, pointer) in transfers {
            if load {
                let value = self.load_memory(pointer, &Width::Word, false);
                if register == Reg::PC {
                    next = Some(self.pc_write(value, true));
                } else {
                    self.store_register(&register, value);
                }
            } else {
                let value = if register == Reg::PC {
                    self.builder
                        .ins()
                        .iconst(types::I32, i64::from(self.pc.wrapping_add(if self.thumb { 4 } else { 12 })))
                } else {
                    self.register(&register)
                };
                self.store_memory(pointer, &Width::Word, value);
            }
        }
        next
    }
}
