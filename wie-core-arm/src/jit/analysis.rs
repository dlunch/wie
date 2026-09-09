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
    let size = if entry.thumb { 2 } else { 4 };
    let mut pending = VecDeque::from([entry.pc]);
    let mut decoded = BTreeMap::new();
    let mut leaders = BTreeSet::from([entry.pc]);
    while let Some(pc) = pending.pop_front() {
        if decoded.len() == 256 {
            break;
        }
        if decoded.contains_key(&pc) || pc % u32::from(size) != 0 {
            continue;
        }
        let Some(offset) = pc.checked_sub(base).map(|offset| offset as usize) else {
            continue;
        };
        let Some(raw) = bytes.get(offset..).and_then(|tail| tail.get(..usize::from(size))) else {
            continue;
        };
        let Some((condition, operation)) = (if entry.thumb {
            decode_thumb(u16::from_le_bytes([raw[0], raw[1]]), pc)
        } else {
            let raw = u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]);
            CONDITIONS.get((raw >> 28) as usize).copied().zip(decode_arm(raw, pc))
        }) else {
            continue;
        };
        let next = pc.wrapping_add(u32::from(size));
        match operation {
            Operation::Branch { target, exchange, .. } => {
                if let Value::Immediate(target) = target
                    && !exchange
                {
                    leaders.insert(target);
                    pending.push_back(target);
                }
                if condition != Condition::Always {
                    leaders.insert(next);
                    pending.push_back(next);
                }
            }
            _ => pending.push_back(next),
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
            let terminates = matches!(instruction.operation, Operation::Branch { .. });
            instructions.push(instruction);
            pc = pc.wrapping_add(u32::from(size));
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
fn decode_thumb(raw: u16, pc: u32) -> Option<(Condition, Operation)> {
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
                Condition::Always,
                Operation::Branch {
                    target: right.value,
                    link,
                    exchange: true,
                },
            ));
        }
        if raw & 0x00c0 == 0 || target == 15 {
            return None;
        }
        op = match kind {
            0 => AluOp::Add,
            1 => AluOp::Sub,
            _ => AluOp::Move,
        };
        destination = if kind == 1 { None } else { Some(target) };
        left = if kind == 2 { Value::Immediate(0) } else { Value::Register(target) };
        set_flags = kind == 1;
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
        return Some((Condition::Always, decode_thumb_memory(raw, pc)));
    } else if raw & 0xf000 == 0xd000 && (raw >> 8) & 15 < 14 {
        let offset = i32::from(raw as u8 as i8) * 2;
        return Some((
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

fn decode_arm(raw: u32, pc: u32) -> Option<Operation> {
    let rd = ((raw >> 12) & 15) as u8;
    let rn = ((raw >> 16) & 15) as u8;
    let rm = (raw & 15) as u8;
    if matches!(raw & 0x0fff_fff0, 0x012f_ff10 | 0x012f_ff30) {
        let link = (raw & 0x20 != 0).then_some(pc.wrapping_add(4));
        if link.is_some() && rm == 15 {
            return None;
        }
        return Some(Operation::Branch {
            target: if rm == 15 {
                Value::Immediate(pc.wrapping_add(8))
            } else {
                Value::Register(rm)
            },
            link,
            exchange: true,
        });
    }
    if raw & 0x0e00_0000 == 0x0a00_0000 {
        let offset = ((raw << 8) as i32) >> 6;
        return Some(Operation::Branch {
            target: Value::Immediate(pc.wrapping_add(8).wrapping_add_signed(offset)),
            link: (raw & 0x0100_0000 != 0).then_some(pc.wrapping_add(4)),
            exchange: false,
        });
    }
    let extra_memory = raw & 0x0e00_0090 == 0x0000_0090 && raw & 0x60 != 0;
    if raw & 0x0c00_0000 == 0x0400_0000 || extra_memory {
        return decode_arm_memory(raw, pc, extra_memory);
    }
    if raw & 0x0c00_0000 != 0 || (raw & 0x0200_0000 == 0 && raw & 0x90 == 0x90) {
        return None;
    }
    let kind = (raw >> 21) & 15;
    let set_flags = raw & 0x0010_0000 != 0;
    let compare = matches!(kind, 8..=11);
    // S=0 test encodings belong to the miscellaneous instruction space, not the ALU.
    if (compare && (!set_flags || rd != 0)) || (!compare && rd == 15) || (matches!(kind, 13 | 15) && rn != 0) {
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
            if rm == 15 || rn == 15 || rs == 15 {
                return None;
            }
            ShiftAmount::Register(rs)
        };
        shifted(value, ((raw >> 5) & 3) as u8, amount)
    };
    Some(Operation::Alu {
        op,
        destination: if compare { None } else { Some(rd) },
        left,
        right,
        set_flags,
    })
}

fn decode_arm_memory(raw: u32, pc: u32, extra: bool) -> Option<Operation> {
    let rd = ((raw >> 12) & 15) as u8;
    let rn = ((raw >> 16) & 15) as u8;
    let rm = (raw & 15) as u8;
    let pre_index = raw & 0x0100_0000 != 0;
    let write_back = !pre_index || raw & 0x0020_0000 != 0;
    let load = raw & 0x0010_0000 != 0;
    // PC destinations/stores and unprivileged accesses have separate execution contracts.
    if rd == 15 || (!pre_index && raw & 0x0020_0000 != 0) || (write_back && (rn == 15 || rn == rd)) {
        return None;
    }
    let width;
    let signed;
    let offset;
    if extra {
        let kind = (raw >> 5) & 3;
        if !load && kind != 1 {
            return None;
        }
        signed = kind >= 2;
        width = if kind == 2 { Width::Byte } else { Width::Half };
        let value = if raw & 0x0040_0000 != 0 {
            Value::Immediate(((raw >> 4) & 0xf0) | (raw & 15))
        } else {
            if raw & 0xf00 != 0 || rm == 15 || (write_back && rn == rm) {
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
    Some(if load {
        Operation::Load {
            destination: rd,
            address,
            width,
            signed,
        }
    } else {
        Operation::Store {
            value: Value::Register(rd),
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
            0xef000000, 0xe1b0f00e, 0xe129f000, 0xe10f0000, 0xf3a00001, 0xe0000090, 0xe6000010, 0xe1a00f11, 0xe4911004, 0xe59ff000, 0xe5c0f000,
            0xe1c000d0, 0xe1900fb1, 0xe4b10004,
        ] {
            assert!(arm(&[opcode]).is_none(), "{opcode:08x}");
            let ir = arm(&[0xe3a00001, opcode, 0xe2800001]).unwrap();
            assert_eq!(ir.blocks.len(), 1, "{opcode:08x}");
            assert_eq!(ir.blocks[0].instructions.len(), 1, "{opcode:08x}");
            assert_eq!(ir.blocks[0].instructions[0].pc, 0x1000, "{opcode:08x}");
        }
        for opcode in [0xdf00, 0xde00, 0xf000, 0xf800, 0xe800, 0xb200, 0x4600, 0x4701, 0xbd00] {
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
