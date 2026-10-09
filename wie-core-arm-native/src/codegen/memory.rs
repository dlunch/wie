use core::mem::{offset_of, size_of};

use cranelift_codegen::ir::{Endianness, InstBuilder, MemFlagsData, Value, condcodes::IntCC, types};

use wie_arm_jit_types::{
    MemoryPage,
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
                let pointer = self.guest_pointer(effective, width);
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
                let pointer = self.guest_pointer(effective, width);
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
                let pointer = self.guest_pointer(address, width);
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
        let (offset, _) = self.operand(&address.offset, false);
        let writeback = if address.subtract {
            self.builder.ins().isub(base, offset)
        } else {
            self.builder.ins().iadd(base, offset)
        };
        (if address.pre_index { writeback } else { base }, writeback)
    }

    fn guest_pointer(&mut self, address: Value, width: &Width) -> Value {
        let page = self.builder.ins().ushr_imm_s(address, 16);
        let page = self.builder.ins().uextend(self.ptr_type, page);
        let offset = self.builder.ins().imul_imm_s(page, size_of::<MemoryPage>() as i64);
        let entry = self.builder.ins().iadd(self.pages, offset);
        let pointer = self
            .builder
            .ins()
            .load(self.ptr_type, MemFlagsData::trusted(), entry, offset_of!(MemoryPage, bytes) as i32);
        let unmapped = self.builder.ins().icmp_imm_s(IntCC::Equal, pointer, 0);
        let next = self.builder.create_block();
        let pc = self.builder.ins().iconst(types::I32, i64::from(self.pc));
        self.builder.ins().brif(unmapped, self.fault, &[pc.into(), address.into()], next, &[]);
        self.builder.switch_to_block(next);
        // Guest accesses are naturally aligned; keep host accesses within the backing page.
        let mask = match width {
            Width::Byte => 0xffff,
            Width::Half => 0xfffe,
            Width::Word => 0xfffc,
        };
        let offset = self.builder.ins().band_imm_s(address, mask);
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
        let count = registers.count_ones();
        if count == 0 {
            return None;
        }
        // At most 16 aligned words span two pages; admit both before any guest access.
        let first_pointer = self.guest_pointer(address, &Width::Word);
        let last_offset = i64::from(count - 1) * 4;
        let last_pointer = if count == 1 {
            first_pointer
        } else {
            let last = self.builder.ins().iadd_imm_s(address, last_offset);
            self.guest_pointer(last, &Width::Word)
        };
        let first_offset = self.builder.ins().band_imm_s(address, 0xfffc);
        let mut next = None;
        for (index, register) in (0..16).filter(|register| registers & (1 << register) != 0).enumerate() {
            let offset = index as i64 * 4;
            let pointer = if index == 0 {
                first_pointer
            } else if offset == last_offset {
                last_pointer
            } else {
                let first = self.builder.ins().iadd_imm_s(first_pointer, offset);
                let last = self.builder.ins().iadd_imm_s(last_pointer, offset - last_offset);
                let crossed = self
                    .builder
                    .ins()
                    .icmp_imm_s(IntCC::UnsignedGreaterThanOrEqual, first_offset, 0x10000 - offset);
                self.builder.ins().select(crossed, last, first)
            };
            let register = Reg::new(register);
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
