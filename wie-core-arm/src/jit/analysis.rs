use alloc::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    vec::Vec,
};

use wie_arm_jit::{Address, AluOp, BasicBlock, Condition, Instruction, Operand, Operation, RegionIr, RegionKey, Shift, ShiftAmount, Value, Width};

const CONDITIONS: [Condition; 15] = [
    Condition::Eq,
    Condition::Ne,
    Condition::Cs,
    Condition::Cc,
    Condition::Mi,
    Condition::Pl,
    Condition::Vs,
    Condition::Vc,
    Condition::Hi,
    Condition::Ls,
    Condition::Ge,
    Condition::Lt,
    Condition::Gt,
    Condition::Le,
    Condition::Always,
];

pub(super) fn analyze(bytes: &[u8], base: u32, entry: RegionKey) -> Option<RegionIr> {
    if bytes.len() > 16 * 1024 || !matches!(entry.cpu_mode, 0x10 | 0x1f) || u64::from(base) + bytes.len() as u64 > 1u64 << 32 {
        return None;
    }
    let alignment = if entry.thumb { 2 } else { 4 };
    let mut pending = VecDeque::from([entry.pc]);
    let mut decoded = BTreeMap::new();
    let mut leaders = BTreeSet::from([entry.pc]);
    while let Some(pc) = pending.pop_front() {
        if decoded.len() == 256 {
            break;
        }
        if decoded.contains_key(&pc) || pc % alignment != 0 {
            continue;
        }
        let Some(offset) = pc.checked_sub(base).map(|offset| offset as usize) else {
            continue;
        };
        let Some(raw) = bytes.get(offset..) else {
            continue;
        };
        let Some((size, condition, operation)) = (if entry.thumb {
            decode_thumb(raw, pc)
        } else {
            raw.first_chunk()
                .and_then(|raw| decode_arm(u32::from_le_bytes(*raw), pc, entry.cpu_mode))
                .map(|(condition, operation)| (4, condition, operation))
        }) else {
            continue;
        };
        let next = pc.wrapping_add(u32::from(size));
        if writes_pc(&operation) {
            if let Operation::Branch {
                target: Value::Immediate(target),
                exchange: false,
                ..
            } = operation
            {
                leaders.insert(target);
                pending.push_back(target);
            }
            if condition != Condition::Always {
                leaders.insert(next);
                pending.push_back(next);
            }
        } else {
            pending.push_back(next);
        }
        decoded.insert(
            pc,
            Instruction {
                pc,
                size,
                condition,
                operation,
            },
        );
    }
    if !decoded.contains_key(&entry.pc) {
        return None;
    }

    // Split after discovery so a backward edge can split an already decoded run.
    let mut blocks = Vec::new();
    for start in core::iter::once(entry.pc).chain(leaders.iter().copied().filter(|pc| *pc != entry.pc)) {
        // Omitted blocks are exact-PC dispatcher exits, not synthetic guest instructions.
        if blocks.len() == 32 {
            break;
        }
        let mut instructions = Vec::new();
        let mut pc = start;
        while let Some(instruction) = decoded.remove(&pc) {
            let terminates = writes_pc(&instruction.operation);
            pc = pc.wrapping_add(u32::from(instruction.size));
            instructions.push(instruction);
            if terminates || leaders.contains(&pc) {
                break;
            }
        }
        if !instructions.is_empty() {
            blocks.push(BasicBlock { instructions });
        }
    }
    Some(RegionIr { entry, blocks })
}

fn writes_pc(operation: &Operation) -> bool {
    match operation {
        Operation::Branch { .. } | Operation::Alu { destination: Some(15), .. } | Operation::Load { destination: 15, .. } => true,
        Operation::MultipleTransfer { registers, load: true, .. } => registers & 0x8000 != 0,
        _ => false,
    }
}

fn shifted(value: Value, kind: u8, amount: ShiftAmount) -> Operand {
    let (shift, amount) = match (kind, amount) {
        (1, ShiftAmount::Immediate(0)) => (Shift::Lsr, ShiftAmount::Immediate(32)),
        (2, ShiftAmount::Immediate(0)) => (Shift::Asr, ShiftAmount::Immediate(32)),
        (3, ShiftAmount::Immediate(0)) => (Shift::Rrx, ShiftAmount::Immediate(1)),
        (0, amount) => (Shift::Lsl, amount),
        (1, amount) => (Shift::Lsr, amount),
        (2, amount) => (Shift::Asr, amount),
        (_, amount) => (Shift::Ror, amount),
    };
    Operand { value, shift, amount }
}

// Encodings are derived from ARM DDI 0100I, chapters A6/A7 (Thumb) and A3/A4/A5 (ARM).
fn decode_thumb(bytes: &[u8], pc: u32) -> Option<(u8, Condition, Operation)> {
    let raw = u16::from_le_bytes(*bytes.first_chunk()?);
    if raw & 0xf800 == 0xf000 {
        let suffix = u16::from_le_bytes(*bytes.get(2..)?.first_chunk()?);
        let exchange = match suffix & 0xf800 {
            0xf800 => false,
            0xe800 if suffix & 1 == 0 => true,
            _ => return None,
        };
        let high_offset = (i32::from(raw & 0x07ff) << 21) >> 9;
        let target = pc
            .wrapping_add(4)
            .wrapping_add_signed(high_offset)
            .wrapping_add(u32::from(suffix & 0x07ff) << 1);
        return Some((
            4,
            Condition::Always,
            Operation::Branch {
                target: Value::Immediate(if exchange { target & !3 } else { target }),
                link: Some(pc.wrapping_add(4) | 1),
                exchange,
            },
        ));
    }
    let rd = (raw & 7) as u8;
    let rm = ((raw >> 3) & 7) as u8;
    let mut op = AluOp::Move;
    let mut destination = Some(rd);
    let mut left = Value::Immediate(0);
    let mut right = Operand {
        value: Value::Register(rm),
        shift: Shift::Lsl,
        amount: ShiftAmount::Immediate(0),
    };
    let mut set_flags = true;
    if raw & 0xf800 == 0x1800 {
        op = if raw & 0x0200 == 0 { AluOp::Add } else { AluOp::Sub };
        left = Value::Register(rm);
        right.value = if raw & 0x0400 == 0 {
            Value::Register(((raw >> 6) & 7) as u8)
        } else {
            Value::Immediate(u32::from((raw >> 6) & 7))
        };
    } else if raw & 0xe000 == 0x2000 {
        let register = ((raw >> 8) & 7) as u8;
        let kind = (raw >> 11) & 3;
        op = match kind {
            0 => AluOp::Move,
            2 => AluOp::Add,
            _ => AluOp::Sub,
        };
        destination = if kind == 1 { None } else { Some(register) };
        left = if kind == 0 { Value::Immediate(0) } else { Value::Register(register) };
        right.value = Value::Immediate(u32::from(raw & 0xff));
    } else if raw & 0xe000 == 0 {
        right = shifted(
            Value::Register(rm),
            ((raw >> 11) & 3) as u8,
            ShiftAmount::Immediate(((raw >> 6) & 31) as u8),
        );
    } else if raw & 0xfc00 == 0x4000 {
        left = Value::Register(rd);
        op = match (raw >> 6) & 15 {
            0 | 8 => AluOp::And,
            1 => AluOp::Xor,
            10 => AluOp::Sub,
            11 => AluOp::Add,
            2 | 3 | 4 | 7 => AluOp::Move,
            5 => AluOp::AddCarry,
            6 => AluOp::SubCarry,
            9 => AluOp::Sub,
            12 => AluOp::Or,
            13 => AluOp::Multiply,
            14 => AluOp::BitClear,
            15 => AluOp::Not,
            _ => return None,
        };
        if matches!((raw >> 6) & 15, 8 | 10 | 11) {
            destination = None;
        }
        match (raw >> 6) & 15 {
            kind @ (2 | 3 | 4 | 7) => {
                let shift = match kind {
                    2 => 0,
                    3 => 1,
                    4 => 2,
                    _ => 3,
                };
                right = shifted(Value::Register(rd), shift, ShiftAmount::Register(rm));
                left = Value::Immediate(0);
            }
            9 | 15 => left = Value::Immediate(0),
            _ => {}
        }
    } else if raw & 0xfc00 == 0x4400 {
        let source = ((raw >> 3) & 15) as u8;
        let target = rd | ((raw >> 4) & 8) as u8;
        right.value = if source == 15 {
            Value::Immediate(pc.wrapping_add(4))
        } else {
            Value::Register(source)
        };
        let kind = (raw >> 8) & 3;
        if kind == 3 {
            let link = (raw & 0x0080 != 0).then_some(pc.wrapping_add(2) | 1);
            if raw & 7 != 0 || (link.is_some() && source == 15) {
                return None;
            }
            return Some((
                2,
                Condition::Always,
                Operation::Branch {
                    target: right.value,
                    link,
                    exchange: true,
                },
            ));
        }
        if raw & 0x00c0 == 0 || (kind == 1 && target == 15) {
            return None;
        }
        op = match kind {
            0 => AluOp::Add,
            1 => AluOp::Sub,
            _ => AluOp::Move,
        };
        destination = if kind == 1 { None } else { Some(target) };
        left = if kind == 2 {
            Value::Immediate(0)
        } else if target == 15 {
            Value::Immediate(pc.wrapping_add(4))
        } else {
            Value::Register(target)
        };
        set_flags = kind == 1;
    } else if raw & 0xf000 == 0xc000 {
        let base = ((raw >> 8) & 7) as u8;
        let registers = raw & 0xff;
        let load = raw & 0x0800 != 0;
        let base_listed = registers & (1 << base) != 0;
        if registers == 0 || (!load && base_listed && registers.trailing_zeros() != u32::from(base)) {
            return None;
        }
        return Some((
            2,
            Condition::Always,
            Operation::MultipleTransfer {
                base,
                registers,
                increment: true,
                before: false,
                write_back: !load || !base_listed,
                load,
            },
        ));
    } else if raw & 0xf600 == 0xb400 {
        let load = raw & 0x0800 != 0;
        let registers = (raw & 0xff) | ((raw & 0x100) << if load { 7 } else { 6 });
        if registers == 0 {
            return None;
        }
        return Some((
            2,
            Condition::Always,
            Operation::MultipleTransfer {
                base: 13,
                registers,
                increment: load,
                before: !load,
                write_back: true,
                load,
            },
        ));
    } else if raw & 0xf000 == 0xa000 || raw & 0xff00 == 0xb000 {
        set_flags = false;
        op = if raw & 0xf000 == 0xb000 && raw & 0x80 != 0 {
            AluOp::Sub
        } else {
            AluOp::Add
        };
        destination = Some(if raw & 0xf000 == 0xb000 { 13 } else { ((raw >> 8) & 7) as u8 });
        left = if raw & 0xf800 == 0xa000 {
            Value::Immediate(pc.wrapping_add(4) & !3)
        } else {
            Value::Register(13)
        };
        right.value = Value::Immediate(u32::from(raw & if raw & 0xf000 == 0xb000 { 0x7f } else { 0xff }) * 4);
    } else if raw & 0xf800 == 0x4800 || raw & 0xf000 == 0x5000 || raw & 0xe000 == 0x6000 || raw & 0xe000 == 0x8000 {
        return Some((2, Condition::Always, decode_thumb_memory(raw, pc)));
    } else if raw & 0xf000 == 0xd000 && (raw >> 8) & 15 < 14 {
        let offset = i32::from(raw as u8 as i8) * 2;
        return Some((
            2,
            CONDITIONS[((raw >> 8) & 15) as usize],
            Operation::Branch {
                target: Value::Immediate(pc.wrapping_add(4).wrapping_add_signed(offset)),
                link: None,
                exchange: false,
            },
        ));
    } else if raw & 0xf800 == 0xe000 {
        let offset = i32::from(((raw & 0x7ff) << 5) as i16) >> 4;
        return Some((
            2,
            Condition::Always,
            Operation::Branch {
                target: Value::Immediate(pc.wrapping_add(4).wrapping_add_signed(offset)),
                link: None,
                exchange: false,
            },
        ));
    } else {
        return None;
    }
    Some((
        2,
        Condition::Always,
        Operation::Alu {
            op,
            destination,
            left,
            right,
            set_flags,
        },
    ))
}

fn decode_thumb_memory(raw: u16, pc: u32) -> Operation {
    let mut register = (raw & 7) as u8;
    let mut base = Value::Register(((raw >> 3) & 7) as u8);
    let load;
    let width;
    let mut signed = false;
    let offset;
    if raw & 0xf000 == 0x5000 {
        let kind = (raw >> 9) & 7;
        load = kind >= 3;
        signed = matches!(kind, 3 | 7);
        width = match kind {
            0 | 4 => Width::Word,
            1 | 5 | 7 => Width::Half,
            _ => Width::Byte,
        };
        offset = Value::Register(((raw >> 6) & 7) as u8);
    } else if raw & 0xe000 == 0x6000 || raw & 0xf000 == 0x8000 {
        load = raw & 0x0800 != 0;
        let scale = if raw & 0xf000 == 0x8000 {
            2
        } else if raw & 0x1000 != 0 {
            1
        } else {
            4
        };
        width = match scale {
            1 => Width::Byte,
            2 => Width::Half,
            _ => Width::Word,
        };
        offset = Value::Immediate(u32::from((raw >> 6) & 31) * scale);
    } else {
        let literal = raw & 0xf800 == 0x4800;
        register = ((raw >> 8) & 7) as u8;
        base = if literal {
            Value::Immediate(pc.wrapping_add(4) & !3)
        } else {
            Value::Register(13)
        };
        load = literal || raw & 0x0800 != 0;
        width = Width::Word;
        offset = Value::Immediate(u32::from(raw & 0xff) * 4);
    }
    let address = Address {
        base,
        offset: Operand {
            value: offset,
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        },
        subtract: false,
        pre_index: true,
        write_back: None,
    };
    if load {
        Operation::Load {
            destination: register,
            address,
            width,
            signed,
        }
    } else {
        Operation::Store {
            value: Value::Register(register),
            address,
            width,
        }
    }
}

fn decode_arm(raw: u32, pc: u32, cpu_mode: u8) -> Option<(Condition, Operation)> {
    let rd = ((raw >> 12) & 15) as u8;
    let rn = ((raw >> 16) & 15) as u8;
    let rm = (raw & 15) as u8;
    if raw >> 28 == 15 {
        if raw & 0x0e00_0000 == 0x0a00_0000 {
            let offset = ((raw << 8) as i32) >> 6;
            return Some((
                Condition::Always,
                Operation::Branch {
                    target: Value::Immediate(pc.wrapping_add(8).wrapping_add_signed(offset).wrapping_add((raw >> 23) & 2) | 1),
                    link: Some(pc.wrapping_add(4)),
                    exchange: true,
                },
            ));
        }
        if raw & 0xfd70_f000 == 0xf550_f000 && (raw & 0x0200_0000 == 0 || (raw & 0x10 == 0 && rm != 15)) {
            return Some((Condition::Always, Operation::Nop));
        }
        return None;
    }
    let condition = CONDITIONS[(raw >> 28) as usize];
    if raw & 0x0fff_0fff == 0x010f_0000 {
        return (rd != 15).then_some((condition, Operation::ReadStatus { destination: rd }));
    }
    let immediate_status = raw & 0x0ff0_f000 == 0x0320_f000;
    if immediate_status || raw & 0x0ff0_fff0 == 0x0120_f000 {
        let fields = rn;
        if fields == 0 || (cpu_mode == 0x1f && fields & 1 != 0) || (!immediate_status && rm == 15) {
            return None;
        }
        let value = if immediate_status {
            Value::Immediate((raw & 0xff).rotate_right(((raw >> 8) & 15) * 2))
        } else {
            Value::Register(rm)
        };
        return Some((
            condition,
            Operation::WriteStatus {
                value,
                mask: if fields & 8 == 0 { 0 } else { 0xf000_0000 },
            },
        ));
    }
    if matches!(raw & 0x0fff_fff0, 0x012f_ff10 | 0x012f_ff30) {
        let link = (raw & 0x20 != 0).then_some(pc.wrapping_add(4));
        if link.is_some() && rm == 15 {
            return None;
        }
        return Some((
            condition,
            Operation::Branch {
                target: if rm == 15 {
                    Value::Immediate(pc.wrapping_add(8))
                } else {
                    Value::Register(rm)
                },
                link,
                exchange: true,
            },
        ));
    }
    if raw & 0x0e00_0000 == 0x0a00_0000 {
        let offset = ((raw << 8) as i32) >> 6;
        return Some((
            condition,
            Operation::Branch {
                target: Value::Immediate(pc.wrapping_add(8).wrapping_add_signed(offset)),
                link: (raw & 0x0100_0000 != 0).then_some(pc.wrapping_add(4)),
                exchange: false,
            },
        ));
    }
    if raw & 0x0fc0_00f0 == 0x0000_0090 {
        // MUL/MLA place the destination in bits 19:16; MLA uses bits 15:12 to accumulate.
        let rs = ((raw >> 8) & 15) as u8;
        if rn == 15 || rm == 15 || rs == 15 {
            return None;
        }
        if raw & 0x0020_0000 != 0 {
            if rd == 15 {
                return None;
            }
            return Some((
                condition,
                Operation::MultiplyAccumulate {
                    destination: rn,
                    left: rm,
                    right: rs,
                    accumulate: rd,
                    set_flags: raw & 0x0010_0000 != 0,
                },
            ));
        }
        if rd != 0 {
            return None;
        }
        return Some((
            condition,
            Operation::Alu {
                op: AluOp::Multiply,
                destination: Some(rn),
                left: Value::Register(rm),
                right: Operand {
                    value: Value::Register(rs),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0),
                },
                set_flags: raw & 0x0010_0000 != 0,
            },
        ));
    }
    if raw & 0x0f80_00f0 == 0x0080_0090 {
        let rs = ((raw >> 8) & 15) as u8;
        if rd == 15 || rn == 15 || rm == 15 || rs == 15 || rd == rn {
            return None;
        }
        return Some((
            condition,
            Operation::MultiplyLong {
                low: rd,
                high: rn,
                left: rm,
                right: rs,
                signed: raw & 0x0040_0000 != 0,
                accumulate: raw & 0x0020_0000 != 0,
                set_flags: raw & 0x0010_0000 != 0,
            },
        ));
    }
    if raw & 0x0fff_0ff0 == 0x016f_0f10 {
        if rd == 15 || rm == 15 {
            return None;
        }
        return Some((
            condition,
            Operation::Alu {
                op: AluOp::CountLeadingZeros,
                destination: Some(rd),
                left: Value::Immediate(0),
                right: Operand {
                    value: Value::Register(rm),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0),
                },
                set_flags: false,
            },
        ));
    }
    if raw & 0x0fb0_0ff0 == 0x0100_0090 {
        if rd == 15 || rn == 15 || rm == 15 || rn == rd || rn == rm {
            return None;
        }
        return Some((
            condition,
            Operation::Swap {
                destination: rd,
                address: rn,
                value: rm,
                width: if raw & 0x0040_0000 == 0 { Width::Word } else { Width::Byte },
            },
        ));
    }
    if raw & 0x0e00_0000 == 0x0800_0000 {
        let registers = raw as u16;
        let write_back = raw & 0x0020_0000 != 0;
        let load = raw & 0x0010_0000 != 0;
        if raw & 0x0040_0000 != 0
            || rn == 15
            || registers == 0
            || (write_back && registers & (1 << rn) != 0 && (load || registers.trailing_zeros() != u32::from(rn)))
        {
            return None;
        }
        return Some((
            condition,
            Operation::MultipleTransfer {
                base: rn,
                registers,
                increment: raw & 0x0080_0000 != 0,
                before: raw & 0x0100_0000 != 0,
                write_back,
                load,
            },
        ));
    }
    let extra_memory = raw & 0x0e00_0090 == 0x0000_0090 && raw & 0x60 != 0;
    if raw & 0x0c00_0000 == 0x0400_0000 || extra_memory {
        return decode_arm_memory(raw, pc, extra_memory).map(|operation| (condition, operation));
    }
    if raw & 0x0c00_0000 != 0 || (raw & 0x0200_0000 == 0 && raw & 0x90 == 0x90) {
        return None;
    }
    let kind = (raw >> 21) & 15;
    let set_flags = raw & 0x0010_0000 != 0;
    let compare = matches!(kind, 8..=11);
    // S=0 test encodings belong to the miscellaneous instruction space, not the ALU.
    if (compare && (!set_flags || rd != 0)) || (rd == 15 && set_flags) || (matches!(kind, 13 | 15) && rn != 0) {
        return None;
    }
    let op = match kind {
        0 | 8 => AluOp::And,
        1 | 9 => AluOp::Xor,
        2 | 10 => AluOp::Sub,
        3 => AluOp::ReverseSub,
        4 | 11 => AluOp::Add,
        5 => AluOp::AddCarry,
        6 => AluOp::SubCarry,
        7 => AluOp::ReverseSubCarry,
        12 => AluOp::Or,
        13 => AluOp::Move,
        14 => AluOp::BitClear,
        _ => AluOp::Not,
    };
    let left = if matches!(kind, 13 | 15) {
        Value::Immediate(0)
    } else if rn == 15 {
        Value::Immediate(pc.wrapping_add(8))
    } else {
        Value::Register(rn)
    };
    let right = if raw & 0x0200_0000 != 0 {
        let rotate = ((raw >> 8) & 15) as u8 * 2;
        Operand {
            value: Value::Immediate(raw & 0xff),
            shift: if rotate == 0 { Shift::Lsl } else { Shift::Ror },
            amount: ShiftAmount::Immediate(rotate),
        }
    } else {
        let value = if rm == 15 {
            Value::Immediate(pc.wrapping_add(8))
        } else {
            Value::Register(rm)
        };
        let amount = if raw & 0x10 == 0 {
            ShiftAmount::Immediate(((raw >> 7) & 31) as u8)
        } else {
            let rs = ((raw >> 8) & 15) as u8;
            if rd == 15 || rm == 15 || rn == 15 || rs == 15 {
                return None;
            }
            ShiftAmount::Register(rs)
        };
        shifted(value, ((raw >> 5) & 3) as u8, amount)
    };
    Some((
        condition,
        Operation::Alu {
            op,
            destination: if compare { None } else { Some(rd) },
            left,
            right,
            set_flags,
        },
    ))
}

fn decode_arm_memory(raw: u32, pc: u32, extra: bool) -> Option<Operation> {
    let rd = ((raw >> 12) & 15) as u8;
    let rn = ((raw >> 16) & 15) as u8;
    let rm = (raw & 15) as u8;
    let pre_index = raw & 0x0100_0000 != 0;
    let write_back = !pre_index || raw & 0x0020_0000 != 0;
    let load = raw & 0x0010_0000 != 0;
    let kind = (raw >> 5) & 3;
    let double_transfer = extra && !load && kind >= 2;
    let user_access = !pre_index && raw & 0x0020_0000 != 0;
    if (extra && user_access)
        || (write_back && (rn == 15 || rn == rd))
        || (rd == 15 && (extra || raw & 0x0040_0000 != 0 || (load && user_access)))
        || (double_transfer && (rd > 12 || rd & 1 != 0 || (write_back && rn == rd + 1)))
    {
        return None;
    }
    let width;
    let signed;
    let offset;
    if extra {
        signed = kind >= 2;
        width = if kind == 2 { Width::Byte } else { Width::Half };
        let value = if raw & 0x0040_0000 != 0 {
            Value::Immediate(((raw >> 4) & 0xf0) | (raw & 15))
        } else {
            if raw & 0xf00 != 0 || rm == 15 || (write_back && rn == rm) || (double_transfer && kind == 2 && (rm == rd || rm == rd + 1)) {
                return None;
            }
            Value::Register(rm)
        };
        offset = Operand {
            value,
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        };
    } else {
        signed = false;
        width = if raw & 0x0040_0000 != 0 { Width::Byte } else { Width::Word };
        offset = if raw & 0x0200_0000 == 0 {
            Operand {
                value: Value::Immediate(raw & 0xfff),
                shift: Shift::Lsl,
                amount: ShiftAmount::Immediate(0),
            }
        } else {
            if raw & 0x10 != 0 || rm == 15 || (write_back && rn == rm) {
                return None;
            }
            shifted(
                Value::Register(rm),
                ((raw >> 5) & 3) as u8,
                ShiftAmount::Immediate(((raw >> 7) & 31) as u8),
            )
        };
    }
    let address = Address {
        base: if rn == 15 {
            Value::Immediate(pc.wrapping_add(8))
        } else {
            Value::Register(rn)
        },
        offset,
        subtract: raw & 0x0080_0000 == 0,
        pre_index,
        write_back: write_back.then_some(rn),
    };
    Some(if double_transfer {
        Operation::DoubleTransfer {
            register: rd,
            address,
            load: kind == 2,
        }
    } else if load {
        Operation::Load {
            destination: rd,
            address,
            width,
            signed,
        }
    } else {
        Operation::Store {
            value: if rd == 15 {
                Value::Immediate(pc.wrapping_add(12))
            } else {
                Value::Register(rd)
            },
            address,
            width,
        }
    })
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;

    const ENTRY: RegionKey = RegionKey {
        pc: 0x1000,
        thumb: true,
        cpu_mode: 0x10,
    };

    fn thumb(code: &[u16], entry: u32) -> Option<RegionIr> {
        let bytes: Vec<_> = code.iter().flat_map(|opcode| opcode.to_le_bytes()).collect();
        analyze(&bytes, 0x1000, RegionKey { pc: entry, ..ENTRY })
    }

    #[test]
    fn thumb_countdown_has_a_backedge_and_conditional_fallthrough() {
        // movs r0,#3; subs r0,#1; cmp r0,#0; bne 0x1002; bx lr
        let ir = thumb(&[0x2003, 0x3801, 0x2800, 0xd1fc, 0x4770], 0x1000).unwrap();
        assert_eq!(ir.blocks.iter().map(|b| b.instructions.len()).collect::<Vec<_>>(), [1, 3, 1]);
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Move,
                destination: Some(0),
                left: Value::Immediate(0),
                right: Operand {
                    value: Value::Immediate(3),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0)
                },
                set_flags: true,
            }
        );
        assert!(matches!(
            ir.blocks[1].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: Some(0),
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[1].instructions[1].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: None,
                set_flags: true,
                ..
            }
        ));
        let branch = &ir.blocks[1].instructions[2];
        assert_eq!(branch.condition, Condition::Ne);
        assert_eq!(
            branch.operation,
            Operation::Branch {
                target: Value::Immediate(0x1002),
                link: None,
                exchange: false
            }
        );
        assert_eq!(
            ir.blocks[2].instructions[0].operation,
            Operation::Branch {
                target: Value::Register(14),
                link: None,
                exchange: true
            }
        );
    }

    #[test]
    fn snapshot_reaches_before_entry_without_decoding_unreachable_bytes() {
        // adds r0,#1; b 0x1000; undefined; b 0x1000
        let ir = thumb(&[0x3001, 0xe7fd, 0xde00, 0xe7fb], 0x1006).unwrap();
        assert_eq!(ir.blocks[0].instructions[0].pc, 0x1006);
        assert_eq!(
            ir.blocks.iter().flat_map(|b| &b.instructions).map(|i| i.pc).collect::<Vec<_>>(),
            [0x1006, 0x1000, 0x1002]
        );
        assert_eq!(
            ir.blocks[1].instructions[1].operation,
            Operation::Branch {
                target: Value::Immediate(0x1000),
                link: None,
                exchange: false
            }
        );
    }

    #[test]
    fn unsupported_entry_is_not_a_region_but_executed_prefix_is_retained() {
        assert!(thumb(&[0xdf00], 0x1000).is_none());
        assert!(thumb(&[0xdf00, 0x2001], 0x1000).is_none());
        assert!(arm(&[0x0f000000, 0xe3a00001]).is_none());
        let ir = thumb(&[0x2001, 0xdf00, 0x3001], 0x1000).unwrap();
        assert_eq!(ir.blocks.len(), 1);
        assert_eq!(ir.blocks[0].instructions.len(), 1);
        assert_eq!(ir.blocks[0].instructions[0].pc, 0x1000);
        let ir = arm(&[0xe3a00001, 0x0f000000, 0xe2800001]).unwrap();
        assert_eq!(ir.blocks.len(), 1);
        assert_eq!(ir.blocks[0].instructions.len(), 1);
        assert_eq!(ir.blocks[0].instructions[0].pc, 0x1000);
    }

    #[test]
    fn unsupported_branch_path_keeps_the_supported_alternative() {
        for (code, expected) in [
            ([0xd001, 0xdf00, 0x3001, 0x3101, 0x4770], [0x1000, 0x1006, 0x1008]),
            ([0xd001, 0x3101, 0x4770, 0xdf00, 0x3001], [0x1000, 0x1002, 0x1004]),
        ] {
            let ir = thumb(&code, 0x1000).unwrap();
            assert_eq!(ir.blocks.len(), 2);
            assert_eq!(ir.blocks.iter().flat_map(|b| &b.instructions).map(|i| i.pc).collect::<Vec<_>>(), expected);
        }
        for (code, expected) in [
            ([0x0a000001, 0xef000000, 0xe2800001, 0xe2811001, 0xe12fff1e], [0x1000, 0x100c, 0x1010]),
            ([0x0a000001, 0xe2811001, 0xe12fff1e, 0xef000000, 0xe2800001], [0x1000, 0x1004, 0x1008]),
        ] {
            let ir = arm(&code).unwrap();
            assert_eq!(ir.blocks.len(), 2);
            assert_eq!(ir.blocks.iter().flat_map(|b| &b.instructions).map(|i| i.pc).collect::<Vec<_>>(), expected);
        }
    }

    #[test]
    fn invalid_or_partial_boundaries_terminate() {
        assert!(analyze(&[], 0x1000, ENTRY).is_none());
        assert!(analyze(&[0x01], 0x1000, ENTRY).is_none());
        assert!(analyze(&[0x01, 0x20], 0x1000, RegionKey { pc: 0x1001, ..ENTRY }).is_none());
        assert!(analyze(&[0x01, 0x20], 0x1000, RegionKey { pc: 0xffe, ..ENTRY }).is_none());
        assert!(analyze(&[0x01, 0x20], 0x1000, RegionKey { cpu_mode: 0x13, ..ENTRY }).is_none());
        let ir = analyze(&[0x01, 0x20, 0xff], 0x1000, ENTRY).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 1);
        assert!(analyze(&vec![0; 16 * 1024 + 1], 0x1000, ENTRY).is_none());
    }

    fn arm(code: &[u32]) -> Option<RegionIr> {
        let bytes: Vec<_> = code.iter().flat_map(|opcode| opcode.to_le_bytes()).collect();
        analyze(&bytes, 0x1000, RegionKey { thumb: false, ..ENTRY })
    }

    #[test]
    fn arm_conditions_comparisons_and_backedges() {
        // mov r0,#3; subs r0,r0,#1; cmp r0,#0; bne 0x1004; bx lr
        let ir = arm(&[0xe3a00003, 0xe2500001, 0xe3500000, 0x1afffffc, 0xe12fff1e]).unwrap();
        assert_eq!(ir.blocks.iter().map(|b| b.instructions.len()).collect::<Vec<_>>(), [1, 3, 1]);
        let instructions = &ir.blocks[1].instructions;
        assert!(matches!(
            instructions[0].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: Some(0),
                set_flags: true,
                ..
            }
        ));
        assert!(matches!(
            instructions[1].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: None,
                set_flags: true,
                ..
            }
        ));
        assert_eq!(instructions[2].condition, Condition::Ne);
        assert_eq!(
            instructions[2].operation,
            Operation::Branch {
                target: Value::Immediate(0x1004),
                link: None,
                exchange: false
            }
        );
        // Conditional ordinary instructions keep their condition without making a branch.
        let ir = arm(&[0x02812001, 0xe12fff1e]).unwrap();
        assert_eq!(ir.blocks.len(), 1);
        assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Eq);
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(2),
                left: Value::Register(1),
                set_flags: false,
                ..
            }
        ));
    }

    #[test]
    fn shifts_preserve_zero_encoding_and_register_amount_semantics() {
        // movs r0,r1,lsr #32; movs r2,r3,asr #32; movs r4,r5,rrx; movs r6,r7,ror r8
        let ir = arm(&[0xe1b00021, 0xe1b02043, 0xe1b04065, 0xe1b06877]).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 4);
        for (instruction, expected) in ir.blocks[0].instructions.iter().zip([
            Operand {
                value: Value::Register(1),
                shift: Shift::Lsr,
                amount: ShiftAmount::Immediate(32),
            },
            Operand {
                value: Value::Register(3),
                shift: Shift::Asr,
                amount: ShiftAmount::Immediate(32),
            },
            Operand {
                value: Value::Register(5),
                shift: Shift::Rrx,
                amount: ShiftAmount::Immediate(1),
            },
            Operand {
                value: Value::Register(7),
                shift: Shift::Ror,
                amount: ShiftAmount::Register(8),
            },
        ]) {
            assert!(matches!(instruction.operation, Operation::Alu { op: AluOp::Move, right, set_flags: true, .. } if right == expected));
        }
        let ir = thumb(&[0x0808, 0x101a, 0x41ce], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 3);
        for (instruction, expected) in ir.blocks[0].instructions.iter().zip([
            Operand {
                value: Value::Register(1),
                shift: Shift::Lsr,
                amount: ShiftAmount::Immediate(32),
            },
            Operand {
                value: Value::Register(3),
                shift: Shift::Asr,
                amount: ShiftAmount::Immediate(32),
            },
            Operand {
                value: Value::Register(6),
                shift: Shift::Ror,
                amount: ShiftAmount::Register(1),
            },
        ]) {
            assert!(matches!(instruction.operation, Operation::Alu { op: AluOp::Move, right, .. } if right == expected));
        }
    }

    #[test]
    fn arm_rotated_immediates_do_not_discard_shifter_carry() {
        let ir = arm(&[0xe3b00102, 0xe3b01002, 0xe3100102, 0xe3300102]).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 4);
        for index in [0, 2, 3] {
            assert!(matches!(
                ir.blocks[0].instructions[index].operation,
                Operation::Alu {
                    right: Operand {
                        value: Value::Immediate(2),
                        shift: Shift::Ror,
                        amount: ShiftAmount::Immediate(2)
                    },
                    ..
                }
            ));
        }
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Alu {
                right: Operand {
                    value: Value::Immediate(2),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0)
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Alu {
                op: AluOp::And,
                destination: None,
                set_flags: true,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Alu {
                op: AluOp::Xor,
                destination: None,
                set_flags: true,
                ..
            }
        ));
    }

    #[test]
    fn pc_operands_are_pipeline_constants_and_thumb_literals_are_aligned() {
        // mov r0,pc; add r1,pc,#4; ldr r2,[pc,#8]
        let ir = arm(&[0xe1a0000f, 0xe28f1004, 0xe59f2008]).unwrap();
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                right: Operand {
                    value: Value::Immediate(0x1008),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Alu {
                left: Value::Immediate(0x100c),
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Load {
                address: wie_arm_jit::Address {
                    base: Value::Immediate(0x1010),
                    ..
                },
                ..
            }
        ));
        // movs r0,#0; ldr r1,[pc,#4]; mov r2,pc; add r3,pc,#4
        let ir = thumb(&[0x2000, 0x4901, 0x467a, 0xa301], 0x1000).unwrap();
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Load {
                address: wie_arm_jit::Address {
                    base: Value::Immediate(0x1004),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Alu {
                right: Operand {
                    value: Value::Immediate(0x1008),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Alu {
                left: Value::Immediate(0x1008),
                ..
            }
        ));
    }

    #[test]
    fn memory_width_sign_index_and_writeback_are_preserved() {
        use wie_arm_jit::{Address, Width};
        // ldrb r0,[r1],#1; str r2,[r3,#-4]!; ldrh r4,[r5,#6]; ldrsb r6,[r7,r8]; strh r9,[r10],#2
        let ir = arm(&[0xe4d10001, 0xe5232004, 0xe1d540b6, 0xe19760d8, 0xe0ca90b2]).unwrap();
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Load {
                destination: 0,
                address: Address {
                    base: Value::Register(1),
                    offset: Operand {
                        value: Value::Immediate(1),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0)
                    },
                    subtract: false,
                    pre_index: false,
                    write_back: Some(1)
                },
                width: Width::Byte,
                signed: false,
            }
        );
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Store {
                value: Value::Register(2),
                width: Width::Word,
                address: Address {
                    subtract: true,
                    pre_index: true,
                    write_back: Some(3),
                    ..
                }
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Load {
                destination: 4,
                width: Width::Half,
                signed: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Load {
                destination: 6,
                width: Width::Byte,
                signed: true,
                address: Address {
                    offset: Operand {
                        value: Value::Register(8),
                        ..
                    },
                    write_back: None,
                    ..
                }
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[4].operation,
            Operation::Store {
                value: Value::Register(9),
                width: Width::Half,
                address: Address {
                    pre_index: false,
                    write_back: Some(10),
                    ..
                }
            }
        ));
        let ir = thumb(&[0x7808, 0x8048, 0x5e88, 0x9001, 0x9802], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 5);
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Load {
                width: Width::Byte,
                signed: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Store { width: Width::Half, .. }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Load {
                width: Width::Half,
                signed: true,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Store {
                address: Address {
                    base: Value::Register(13),
                    ..
                },
                width: Width::Word,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[4].operation,
            Operation::Load {
                address: Address {
                    base: Value::Register(13),
                    ..
                },
                width: Width::Word,
                ..
            }
        ));
    }

    #[test]
    fn branches_outside_snapshot_and_dynamic_exits_keep_exact_instruction() {
        let ir = thumb(&[0xd100, 0x4770], 0x1000).unwrap();
        assert_eq!(ir.blocks.len(), 2);
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Branch {
                target: Value::Immediate(0x1004),
                link: None,
                exchange: false
            }
        );
        assert_eq!(ir.blocks[1].instructions[0].pc, 0x1002);
        let ir = arm(&[0xeb000010]).unwrap();
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Branch {
                target: Value::Immediate(0x1048),
                link: Some(0x1004),
                exchange: false
            }
        );
        let ir = arm(&[0x012fff1e, 0xe3a00001]).unwrap();
        assert_eq!(ir.blocks.len(), 2);
        assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Eq);
        assert_eq!(ir.blocks[1].instructions[0].pc, 0x1004);
    }

    #[test]
    fn instruction_and_block_limits_leave_dispatchable_boundaries() {
        let ir = thumb(&[0x3001; 300], 0x1000).unwrap();
        assert_eq!(ir.blocks.len(), 1);
        assert_eq!(ir.blocks[0].instructions.len(), 256);
        assert_eq!(ir.blocks[0].instructions[255].pc, 0x11fe);
        let ir = thumb(&[0xd1ff; 100], 0x1000).unwrap();
        assert_eq!(ir.blocks.len(), 32);
        for (index, block) in ir.blocks.iter().enumerate() {
            assert_eq!(block.instructions.len(), 1);
            assert_eq!(block.instructions[0].pc, 0x1000 + index as u32 * 2);
        }
        // Entry must survive the block cap even when branches reach many earlier blocks.
        let mut code = vec![0xd1ff; 100];
        code.push(0xe79a); // b 0x1000 at 0x10c8
        let ir = thumb(&code, 0x10c8).unwrap();
        assert_eq!(ir.blocks.len(), 32);
        assert_eq!(ir.blocks[0].instructions[0].pc, 0x10c8);
    }

    #[test]
    fn exceptions_newer_isa_and_unpredictable_encodings_stay_in_interpreter() {
        for opcode in [
            0xef000000, 0xe1b0f00e, 0xf3a00001, 0xe6000010, 0xe1a00f11, 0xe4911004, 0xe5c0f000, 0xe1900fb1,
        ] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
            let ir = arm(&[0xe3a00001, opcode, 0xe2800001]).unwrap();
            assert_eq!(ir.blocks.len(), 1, "{opcode:08x}");
            assert_eq!(ir.blocks[0].instructions.len(), 1, "{opcode:08x}");
            assert_eq!(ir.blocks[0].instructions[0].pc, 0x1000, "{opcode:08x}");
        }
        for opcode in [0xdf00, 0xde00, 0xf000, 0xf800, 0xe800, 0xb200, 0x4600, 0x4701] {
            assert!(thumb(&[opcode], 0x1000).is_none(), "{opcode:04x}");
        }
    }

    #[test]
    fn conditional_diamond_rejoins_one_shared_block() {
        // cmp r0,#0; beq 0x1008; movs r1,#1; b 0x100a; movs r1,#2; adds r2,r1,#3; bx lr
        let ir = thumb(&[0x2800, 0xd001, 0x2101, 0xe000, 0x2102, 0x1cca, 0x4770], 0x1000).unwrap();
        assert_eq!(ir.blocks.iter().map(|b| b.instructions.len()).collect::<Vec<_>>(), [2, 2, 1, 2]);
        assert_eq!(
            ir.blocks.iter().map(|b| b.instructions[0].pc).collect::<Vec<_>>(),
            [0x1000, 0x1004, 0x1008, 0x100a]
        );
        assert_eq!(
            ir.blocks[0].instructions[1].operation,
            Operation::Branch {
                target: Value::Immediate(0x1008),
                link: None,
                exchange: false
            }
        );
        assert_eq!(
            ir.blocks[1].instructions[1].operation,
            Operation::Branch {
                target: Value::Immediate(0x100a),
                link: None,
                exchange: false
            }
        );
        assert_eq!(
            ir.blocks[3].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(2),
                left: Value::Register(1),
                right: Operand {
                    value: Value::Immediate(3),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0)
                },
                set_flags: true,
            }
        );
    }

    #[test]
    fn arm_multiply_preserves_conditions_operands_and_flag_selection() {
        for (condition_bits, condition) in CONDITIONS.into_iter().enumerate() {
            for set_flags in [false, true] {
                for (rd, rm, rs) in [(0, 0, 1), (2, 1, 2), (14, 13, 12), (0, 0, 0)] {
                    let opcode = ((condition_bits as u32) << 28)
                        | 0x0000_0090
                        | (u32::from(set_flags) << 20)
                        | (u32::from(rd) << 16)
                        | (u32::from(rs) << 8)
                        | u32::from(rm);
                    let ir = arm(&[opcode, 0xe2800001]).unwrap();
                    assert_eq!(ir.blocks.len(), 1, "{opcode:08x}");
                    assert_eq!(ir.blocks[0].instructions.len(), 2, "{opcode:08x}");
                    assert_eq!(ir.blocks[0].instructions[1].pc, 0x1004);
                    assert_eq!(
                        ir.blocks[0].instructions[0],
                        Instruction {
                            pc: 0x1000,
                            size: 4,
                            condition,
                            operation: Operation::Alu {
                                op: AluOp::Multiply,
                                destination: Some(rd),
                                left: Value::Register(rm),
                                right: Operand {
                                    value: Value::Register(rs),
                                    shift: Shift::Lsl,
                                    amount: ShiftAmount::Immediate(0),
                                },
                                set_flags,
                            },
                        },
                        "{opcode:08x}"
                    );
                }
            }
        }
    }

    #[test]
    fn thumb_multiply_preserves_all_low_register_operands_and_sets_flags() {
        for rd in 0..8 {
            for rm in 0..8 {
                let opcode = 0x4340 | (u16::from(rm) << 3) | u16::from(rd);
                let ir = thumb(&[opcode, 0x3001], 0x1000).unwrap();
                assert_eq!(ir.blocks.len(), 1, "{opcode:04x}");
                assert_eq!(ir.blocks[0].instructions.len(), 2, "{opcode:04x}");
                assert_eq!(ir.blocks[0].instructions[1].pc, 0x1002);
                assert_eq!(
                    ir.blocks[0].instructions[0],
                    Instruction {
                        pc: 0x1000,
                        size: 2,
                        condition: Condition::Always,
                        operation: Operation::Alu {
                            op: AluOp::Multiply,
                            destination: Some(rd),
                            left: Value::Register(rd),
                            right: Operand {
                                value: Value::Register(rm),
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            set_flags: true,
                        },
                    },
                    "{opcode:04x}"
                );
            }
        }
    }

    #[test]
    fn arm_multiply_rejects_pc_and_reserved_bits() {
        for opcode in [
            0xe00f0190, // MUL with PC destination.
            0xe000019f, // MUL with PC as Rm.
            0xe0000f91, // MUL with PC as Rs.
            0xe0001190, // MUL with nonzero SBZ field.
            0xf0000190, // MUL is not an unconditional-space instruction.
        ] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
            let ir = arm(&[0xe3a00001, opcode, 0xe2800001]).unwrap();
            assert_eq!(ir.blocks[0].instructions.len(), 1, "{opcode:08x}");
        }
    }

    #[test]
    fn arm_multiply_accumulate_keeps_all_inputs_and_flag_selection() {
        for set_flags in [false, true] {
            for (destination, left, right, accumulate) in [(2, 1, 3, 4), (2, 2, 2, 2), (14, 13, 12, 11)] {
                let opcode = 0x0020_0090
                    | (u32::from(set_flags) << 20)
                    | (u32::from(destination) << 16)
                    | (u32::from(accumulate) << 12)
                    | (u32::from(right) << 8)
                    | u32::from(left);
                let ir = arm(&[opcode]).unwrap();
                assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Eq);
                assert_eq!(
                    ir.blocks[0].instructions[0].operation,
                    Operation::MultiplyAccumulate {
                        destination,
                        left,
                        right,
                        accumulate,
                        set_flags,
                    }
                );
            }
        }
        for field in [0, 8, 12, 16] {
            assert!(arm(&[0xe0200090 | (15 << field)]).is_none());
        }
    }

    #[test]
    fn arm_long_multiply_keeps_signed_accumulate_and_distinct_destinations() {
        for signed in [false, true] {
            for accumulate in [false, true] {
                for set_flags in [false, true] {
                    for (low, high, left, right) in [(2, 3, 4, 5), (0, 1, 0, 1), (13, 14, 14, 13)] {
                        let opcode = 0x1080_0090
                            | (u32::from(signed) << 22)
                            | (u32::from(accumulate) << 21)
                            | (u32::from(set_flags) << 20)
                            | (u32::from(high) << 16)
                            | (u32::from(low) << 12)
                            | (u32::from(right) << 8)
                            | u32::from(left);
                        let ir = arm(&[opcode]).unwrap();
                        assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Ne);
                        assert_eq!(
                            ir.blocks[0].instructions[0].operation,
                            Operation::MultiplyLong {
                                low,
                                high,
                                left,
                                right,
                                signed,
                                accumulate,
                                set_flags,
                            }
                        );
                    }
                }
            }
        }
        for opcode in [0xe0800090, 0xe08f0293, 0xe081f293, 0xe0810f93, 0xe081029f, 0xe0410293] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn arm_clz_reads_one_register_without_setting_flags() {
        for (rd, rm) in [(0, 1), (14, 14)] {
            let opcode = 0xe16f0f10 | (u32::from(rd) << 12) | u32::from(rm);
            let ir = arm(&[opcode]).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Alu {
                    op: AluOp::CountLeadingZeros,
                    destination: Some(rd),
                    left: Value::Immediate(0),
                    right: Operand {
                        value: Value::Register(rm),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    set_flags: false,
                }
            );
        }
        for opcode in [0xe16fff10, 0xe16f0f1f, 0xe1600f10, 0xe16f0010, 0xe16f0f30, 0xe17f0f10] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn arm_status_access_preserves_user_flags_and_system_control_frontiers() {
        for cpu_mode in [0x10, 0x1f] {
            let entry = RegionKey {
                thumb: false,
                cpu_mode,
                ..ENTRY
            };
            for destination in [0, 14] {
                let bytes = (0x010f0000 | (u32::from(destination) << 12)).to_le_bytes();
                let ir = analyze(&bytes, 0x1000, entry).unwrap();
                assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Eq);
                assert_eq!(ir.blocks[0].instructions[0].operation, Operation::ReadStatus { destination });
            }
            for fields in 1..16 {
                for (opcode, value) in [(0xe120f003u32, Value::Register(3)), (0xe320f4ff, Value::Immediate(0xff000000))] {
                    let bytes = (opcode | (fields << 16)).to_le_bytes();
                    let ir = analyze(&bytes, 0x1000, entry);
                    if cpu_mode == 0x1f && fields & 1 != 0 {
                        assert!(ir.is_none());
                    } else {
                        assert_eq!(
                            ir.unwrap().blocks[0].instructions[0].operation,
                            Operation::WriteStatus {
                                value,
                                mask: if fields & 8 == 0 { 0 } else { 0xf0000000 },
                            }
                        );
                    }
                }
            }
            for opcode in [
                0xe14f0000u32,
                0xe16ff003,
                0xe36ff4ff, // SPSR.
                0xe10ff000,
                0xe128f00f, // PC operands.
                0xe1000000,
                0xe10f0001,
                0xe1280003,
                0xe128f013, // Reserved fields.
                0xe320f001, // Newer hint encoding, not an ARMv5 flags write.
            ] {
                assert!(analyze(&opcode.to_le_bytes(), 0x1000, entry).is_none(), "{opcode:08x}");
            }
        }
    }

    #[test]
    fn arm_unconditional_space_accepts_only_blx_and_valid_prefetch_hints() {
        for (opcode, target) in [(0xfa000000, 0x1009), (0xfb000000, 0x100b), (0xfaffffff, 0x1005)] {
            let ir = arm(&[opcode, 0xe2800001]).unwrap();
            assert_eq!(ir.blocks.len(), 1);
            assert_eq!(ir.blocks[0].instructions.len(), 1);
            assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Always);
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Branch {
                    target: Value::Immediate(target),
                    link: Some(0x1004),
                    exchange: true,
                }
            );
        }
        for opcode in [0xf5d1f004, 0xf551ffff, 0xf5dff000, 0xf7d1f002, 0xf751f062] {
            let ir = arm(&[opcode, 0xe2800001]).unwrap();
            assert_eq!(ir.blocks[0].instructions.len(), 2);
            assert_eq!(ir.blocks[0].instructions[0].condition, Condition::Always);
            assert_eq!(ir.blocks[0].instructions[0].operation, Operation::Nop);
        }
        for opcode in [
            0xf3a00001, 0xf5d1e004, 0xf4d1f004, 0xf5f1f004, 0xf7d1f012, 0xf7d1f00f, 0xf57ff01f, 0xee070f15,
        ] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn thumb_call_pairs_have_one_size_four_instruction_and_no_suffix_entry() {
        for (prefix, suffix, target, exchange) in [
            (0xf000, 0xf800, 0x1004, false),
            (0xf000, 0xf802, 0x1008, false),
            (0xf7ff, 0xfffe, 0x1000, false),
            (0xf000, 0xe802, 0x1008, true),
        ] {
            let ir = thumb(&[prefix, suffix], 0x1000).unwrap();
            assert_eq!(ir.blocks.len(), 1);
            assert_eq!(ir.blocks[0].instructions.len(), 1);
            assert_eq!(
                ir.blocks[0].instructions[0],
                Instruction {
                    pc: 0x1000,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Branch {
                        target: Value::Immediate(target),
                        link: Some(0x1005),
                        exchange,
                    },
                }
            );
            assert!(thumb(&[prefix, suffix], 0x1002).is_none());
        }
        // The conditional branch targets the BL suffix before the prefix is discovered.
        let ir = thumb(&[0xd000, 0xf000, 0xf801, 0x3001, 0x4770], 0x1000).unwrap();
        assert_eq!(
            ir.blocks.iter().flat_map(|b| &b.instructions).map(|i| i.pc).collect::<Vec<_>>(),
            [0x1000, 0x1002, 0x1008]
        );
        let ir = thumb(&[0x3001, 0xf000, 0xe800], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 2);
        assert_eq!(
            ir.blocks[0].instructions[1].operation,
            Operation::Branch {
                target: Value::Immediate(0x1004),
                link: Some(0x1007),
                exchange: true,
            }
        );
        for suffix in [0xe801, 0xf000, 0x3001] {
            assert!(thumb(&[0xf000, suffix], 0x1000).is_none());
        }
        for bytes in [&[0x00, 0xf0][..], &[0x00, 0xf0, 0x00][..]] {
            assert!(analyze(bytes, 0x1000, ENTRY).is_none());
        }
        let ir = thumb(&[0x3001, 0xf000], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn ordinary_alu_pc_writes_terminate_only_the_taken_path() {
        for (opcode, op, left, right) in [
            (0xe1a0f001, AluOp::Move, Value::Immediate(0), Value::Register(1)),
            (0xe08ff001, AluOp::Add, Value::Immediate(0x1008), Value::Register(1)),
        ] {
            for conditional in [false, true] {
                let opcode = if conditional { opcode & 0x0fffffff } else { opcode };
                let ir = arm(&[opcode, 0xe2800001]).unwrap();
                assert_eq!(ir.blocks.len(), if conditional { 2 } else { 1 });
                assert_eq!(ir.blocks[0].instructions.len(), 1);
                assert_eq!(
                    ir.blocks[0].instructions[0].operation,
                    Operation::Alu {
                        op,
                        destination: Some(15),
                        left,
                        right: Operand {
                            value: right,
                            shift: Shift::Lsl,
                            amount: ShiftAmount::Immediate(0)
                        },
                        set_flags: false,
                    }
                );
                if conditional {
                    assert_eq!(ir.blocks[1].instructions[0].pc, 0x1004);
                }
            }
        }
        for (opcode, op, left, right) in [
            (0x4487, AluOp::Add, Value::Immediate(0x1004), Value::Register(0)),
            (0x468f, AluOp::Move, Value::Immediate(0), Value::Register(1)),
            (0x44ff, AluOp::Add, Value::Immediate(0x1004), Value::Immediate(0x1004)),
        ] {
            let ir = thumb(&[opcode, 0x3001], 0x1000).unwrap();
            assert_eq!(ir.blocks.len(), 1);
            assert_eq!(ir.blocks[0].instructions.len(), 1);
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Alu {
                    op,
                    destination: Some(15),
                    left,
                    right: Operand {
                        value: right,
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0)
                    },
                    set_flags: false,
                }
            );
        }
        for opcode in [0xe1b0f001, 0xe1a0f211, 0xe08ff211, 0xe1a0021f, 0xe1a00f11] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
        assert!(thumb(&[0x4587], 0x1000).is_none());
    }

    #[test]
    fn arm_multiple_transfers_preserve_modes_masks_and_base_alias_rules() {
        for (bits, condition) in CONDITIONS.into_iter().enumerate() {
            for increment in [false, true] {
                for before in [false, true] {
                    for write_back in [false, true] {
                        for load in [false, true] {
                            let opcode = ((bits as u32) << 28)
                                | 0x08015018
                                | (u32::from(increment) << 23)
                                | (u32::from(before) << 24)
                                | (u32::from(write_back) << 21)
                                | (u32::from(load) << 20);
                            let ir = arm(&[opcode]).unwrap();
                            assert_eq!(ir.blocks[0].instructions[0].condition, condition);
                            assert_eq!(
                                ir.blocks[0].instructions[0].operation,
                                Operation::MultipleTransfer {
                                    base: 1,
                                    registers: 0x5018,
                                    increment,
                                    before,
                                    write_back,
                                    load
                                }
                            );
                        }
                    }
                }
            }
        }
        for (opcode, base, registers, write_back, load) in [
            (0x28b15018, 1, 0x5018, true, true),
            (0x28a0500c, 0, 0x500c, true, false),
            (0xe890ffff, 0, 0xffff, false, true),
            (0xe8a0ffff, 0, 0xffff, true, false),
            (0xe8800001, 0, 1, false, false),
            (0xe8900001, 0, 1, false, true),
        ] {
            let ir = arm(&[opcode]).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::MultipleTransfer {
                    base,
                    registers,
                    increment: true,
                    before: false,
                    write_back,
                    load
                }
            );
        }
        for opcode in [0xe8910000, 0xe89f0001, 0xe8d10001, 0xe8b10002, 0xe8a10003, 0xf8910001] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn thumb_multiple_transfers_normalize_stack_and_listed_base_writeback() {
        for (opcode, base, registers, increment, before, write_back, load) in [
            (0xc004, 0, 4, true, false, true, false),
            (0xc102, 1, 2, true, false, true, false),
            (0xc901, 1, 1, true, false, true, true),
            (0xc903, 1, 3, true, false, false, true),
            (0xb500, 13, 0x4000, false, true, true, false),
            (0xb5ff, 13, 0x40ff, false, true, true, false),
            (0xbcff, 13, 0x00ff, true, false, true, true),
            (0xbd00, 13, 0x8000, true, false, true, true),
            (0xbdff, 13, 0x80ff, true, false, true, true),
        ] {
            let ir = thumb(&[opcode], 0x1000).unwrap();
            assert_eq!(ir.blocks[0].instructions[0].size, 2);
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::MultipleTransfer {
                    base,
                    registers,
                    increment,
                    before,
                    write_back,
                    load
                }
            );
        }
        for opcode in [0xc000, 0xc800, 0xc103, 0xb400, 0xbc00] {
            assert!(thumb(&[opcode], 0x1000).is_none(), "{opcode:04x}");
        }
    }

    #[test]
    fn memory_pc_loads_terminate_only_the_taken_path_and_word_stores_use_pc_plus_twelve() {
        for opcode in [0xe591f000, 0xe8918001, 0xe8b18001] {
            for conditional in [false, true] {
                let opcode = if conditional { opcode & 0x0fffffff } else { opcode };
                let ir = arm(&[opcode, 0xe2800001]).unwrap();
                assert_eq!(ir.blocks.len(), if conditional { 2 } else { 1 });
                assert_eq!(ir.blocks[0].instructions.len(), 1);
                if conditional {
                    assert_eq!(ir.blocks[1].instructions[0].pc, 0x1004);
                }
            }
        }
        let ir = thumb(&[0xbd00, 0x3001], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 1);
        for opcode in [0xe581f004, 0xe4a1f004] {
            assert!(matches!(
                arm(&[opcode]).unwrap().blocks[0].instructions[0].operation,
                Operation::Store {
                    value: Value::Immediate(0x100c),
                    width: Width::Word,
                    ..
                }
            ));
        }
        for opcode in [0xe5d1f000, 0xe5c1f000, 0xe1c1f0b0, 0xe1d1f0b0, 0xe4b1f004] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn arm_user_access_forms_share_single_transfer_addressing() {
        for cpu_mode in [0x10, 0x1f] {
            for (normal, translated) in [
                (0xe4912004u32, 0xe4b12004u32),
                (0xe4d12004, 0xe4f12004),
                (0xe4812004, 0xe4a12004),
                (0xe4c12004, 0xe4e12004),
                (0xe6912083, 0xe6b12083),
            ] {
                let entry = RegionKey {
                    cpu_mode,
                    thumb: false,
                    ..ENTRY
                };
                let expected = analyze(&normal.to_le_bytes(), 0x1000, entry).unwrap();
                let actual = analyze(&translated.to_le_bytes(), 0x1000, entry).unwrap();
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn arm_swap_preserves_width_and_input_aliases() {
        for (opcode, destination, value, width) in [
            (0xe1012093, 2, 3, Width::Word),
            (0xe1412093, 2, 3, Width::Byte),
            (0xe1012092, 2, 2, Width::Word),
        ] {
            assert_eq!(
                arm(&[opcode]).unwrap().blocks[0].instructions[0].operation,
                Operation::Swap {
                    destination,
                    address: 1,
                    value,
                    width
                }
            );
        }
        for opcode in [0xe101f093, 0xe10f2093, 0xe101209f, 0xe1011093, 0xe1012091, 0xe1012193, 0xe1112093] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn arm_double_transfers_keep_pair_addressing_and_register_restrictions() {
        for load in [false, true] {
            for (pre_index, write_back) in [(true, false), (true, true), (false, true)] {
                for subtract in [false, true] {
                    for (immediate, bits, value) in [(true, 0xa0b, Value::Immediate(0xab)), (false, 4, Value::Register(4))] {
                        let opcode = 0xe0072090u32
                            | (u32::from(pre_index) << 24)
                            | (u32::from(!subtract) << 23)
                            | (u32::from(immediate) << 22)
                            | (u32::from(pre_index && write_back) << 21)
                            | (if load { 2 } else { 3 } << 5)
                            | bits;
                        assert_eq!(
                            arm(&[opcode]).unwrap().blocks[0].instructions[0].operation,
                            Operation::DoubleTransfer {
                                register: 2,
                                address: Address {
                                    base: Value::Register(7),
                                    offset: Operand {
                                        value,
                                        shift: Shift::Lsl,
                                        amount: ShiftAmount::Immediate(0)
                                    },
                                    subtract,
                                    pre_index,
                                    write_back: write_back.then_some(7),
                                },
                                load,
                            },
                            "{opcode:08x}"
                        );
                    }
                }
            }
        }
        for opcode in [0xe1c000d0, 0xe1cf20d0, 0xe18720f2] {
            assert!(matches!(
                arm(&[opcode]).unwrap().blocks[0].instructions[0].operation,
                Operation::DoubleTransfer { .. }
            ));
        }
        for opcode in [
            0xe1c030d0, 0xe1c0e0d0, 0xe1c030f0, 0xe1c0e0f0, // Odd pair or PC.
            0xe1e220d0, 0xe1e320d0, 0xe1e220f0, 0xe1e320f0, // Writeback aliases either pair register.
            0xe18720d2, 0xe18720d3, // LDRD index aliases either destination.
            0xe18721d4, 0xe18720df, 0xe0e720d0, // SBZ, PC index, reserved P/W.
        ] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
        }
    }

    #[test]
    fn arm_data_processing_opcode_families_preserve_operands() {
        for (opcode, op) in [
            (0xe0112003, AluOp::And),
            (0xe0312003, AluOp::Xor),
            (0xe0512003, AluOp::Sub),
            (0xe0712003, AluOp::ReverseSub),
            (0xe0912003, AluOp::Add),
            (0xe0b12003, AluOp::AddCarry),
            (0xe0d12003, AluOp::SubCarry),
            (0xe0f12003, AluOp::ReverseSubCarry),
            (0xe1912003, AluOp::Or),
            (0xe1d12003, AluOp::BitClear),
        ] {
            let ir = arm(&[opcode]).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Alu {
                    op,
                    destination: Some(2),
                    left: Value::Register(1),
                    right: Operand {
                        value: Value::Register(3),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0)
                    },
                    set_flags: true,
                },
                "{opcode:08x}"
            );
        }
        let ir = arm(&[0xe1f02003, 0xe1710003]).unwrap();
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Not,
                destination: Some(2),
                left: Value::Immediate(0),
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: None,
                set_flags: true,
                ..
            }
        ));
    }

    #[test]
    fn thumb_register_arithmetic_and_stack_adjustment() {
        for (opcode, op) in [
            (0x4008, AluOp::And),
            (0x4048, AluOp::Xor),
            (0x4148, AluOp::AddCarry),
            (0x4188, AluOp::SubCarry),
            (0x4308, AluOp::Or),
            (0x4388, AluOp::BitClear),
        ] {
            let ir = thumb(&[opcode], 0x1000).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Alu {
                    op,
                    destination: Some(0),
                    left: Value::Register(0),
                    right: Operand {
                        value: Value::Register(1),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0)
                    },
                    set_flags: true,
                },
                "{opcode:04x}"
            );
        }
        // adds r2,r0,r1; subs r2,r0,r1; subs r2,r0,#3; movs r2,r0 (ADD #0)
        let ir = thumb(&[0x1842, 0x1a42, 0x1ec2, 0x1c02], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 4);
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Add,
                right: Operand {
                    value: Value::Register(1),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Alu {
                op: AluOp::Sub,
                right: Operand {
                    value: Value::Register(1),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Alu {
                op: AluOp::Sub,
                right: Operand {
                    value: Value::Immediate(3),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Alu {
                op: AluOp::Add,
                right: Operand {
                    value: Value::Immediate(0),
                    ..
                },
                set_flags: true,
                ..
            }
        ));
        let ir = thumb(&[0xb004, 0xb084, 0xa804, 0x4680, 0x4488, 0x4588], 0x1000).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 6);
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(13),
                left: Value::Register(13),
                set_flags: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[1].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: Some(13),
                set_flags: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[2].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(0),
                left: Value::Register(13),
                set_flags: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[3].operation,
            Operation::Alu {
                op: AluOp::Move,
                destination: Some(8),
                set_flags: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[4].operation,
            Operation::Alu {
                op: AluOp::Add,
                destination: Some(8),
                left: Value::Register(8),
                set_flags: false,
                ..
            }
        ));
        assert!(matches!(
            ir.blocks[0].instructions[5].operation,
            Operation::Alu {
                op: AluOp::Sub,
                destination: None,
                left: Value::Register(8),
                set_flags: true,
                ..
            }
        ));
    }

    #[test]
    fn branch_displacements_and_pipeline_values_wrap_at_32_bits() {
        for (opcode, target) in [(0xd080, 0x0f04), (0xe400, 0x0804), (0xe3ff, 0x1802), (0xe7fe, 0x1000)] {
            let ir = thumb(&[opcode], 0x1000).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Branch {
                    target: Value::Immediate(target),
                    link: None,
                    exchange: false
                }
            );
        }
        for (opcode, target) in [(0xea800000, 0xfe001008), (0xea7fffff, 0x02001004), (0xeafffffe, 0x1000)] {
            let ir = arm(&[opcode]).unwrap();
            assert_eq!(
                ir.blocks[0].instructions[0].operation,
                Operation::Branch {
                    target: Value::Immediate(target),
                    link: None,
                    exchange: false
                }
            );
        }
        let key = RegionKey {
            pc: 0xffff_fffc,
            thumb: false,
            ..ENTRY
        };
        let ir = analyze(&0xe1a0000fu32.to_le_bytes(), key.pc, key).unwrap();
        assert_eq!(ir.blocks[0].instructions.len(), 1);
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                right: Operand {
                    value: Value::Immediate(4),
                    ..
                },
                ..
            }
        ));
        let key = RegionKey { pc: 0xffff_fffe, ..ENTRY };
        let ir = analyze(&0x4678u16.to_le_bytes(), key.pc, key).unwrap();
        assert!(matches!(
            ir.blocks[0].instructions[0].operation,
            Operation::Alu {
                right: Operand {
                    value: Value::Immediate(2),
                    ..
                },
                ..
            }
        ));
        assert!(analyze(&[0; 4], key.pc, key).is_none());
        assert!(analyze(&[0; 4], 0, key).is_none());
        assert!(analyze(&[0; 3], 0x1000, RegionKey { thumb: false, ..ENTRY }).is_none());
        assert!(
            analyze(
                &[0; 4],
                0x1000,
                RegionKey {
                    pc: 0x1002,
                    thumb: false,
                    ..ENTRY
                }
            )
            .is_none()
        );
        assert!(analyze(&[0; 2], 0x1000, RegionKey { cpu_mode: 0x1f, ..ENTRY }).is_some());
    }

    #[test]
    fn branch_exchange_links_and_pc_sources_are_exact() {
        let ir = arm(&[0xe12fff33]).unwrap();
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Branch {
                target: Value::Register(3),
                link: Some(0x1004),
                exchange: true
            }
        );
        let ir = thumb(&[0x4798], 0x1000).unwrap();
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Branch {
                target: Value::Register(3),
                link: Some(0x1003),
                exchange: true
            }
        );
        let ir = arm(&[0xe12fff1f]).unwrap();
        assert_eq!(
            ir.blocks[0].instructions[0].operation,
            Operation::Branch {
                target: Value::Immediate(0x1008),
                link: None,
                exchange: true
            }
        );
        assert!(arm(&[0xe12fff3f]).is_none());
        assert!(thumb(&[0x47f8], 0x1000).is_none());
    }
}
