extern crate alloc;

use std::{
    io::Write,
    process::{Command, Stdio},
};

use wasm_encoder::{
    CodeSection, ExportKind, ExportSection, Function, FunctionSection, MemArg, MemorySection, MemoryType, Module, TypeSection, ValType,
};
use wie_arm_jit::*;

#[path = "../src/codegen.rs"]
mod codegen;

fn alu(pc: u32, op: AluOp, destination: Option<u8>, left: Value, right: Value, set_flags: bool) -> Instruction {
    Instruction {
        pc,
        size: 2,
        condition: Condition::Always,
        operation: Operation::Alu {
            op,
            destination,
            left,
            right: Operand {
                value: right,
                shift: Shift::Lsl,
                amount: ShiftAmount::Immediate(0),
            },
            set_flags,
        },
    }
}

fn request(instructions: Vec<Instruction>) -> CompileRequest {
    CompileRequest {
        session: 11,
        request: 22,
        regions: vec![CompileRegion {
            ir: RegionIr {
                entry: RegionKey {
                    pc: 0x1000,
                    thumb: true,
                    cpu_mode: 0x10,
                },
                blocks: vec![BasicBlock { instructions }],
            },
            source: vec![CodePageStamp { page: 0, version: 7 }],
            expected_old: Some(CompiledHandle { slot: 3, generation: 9 }),
        }],
    }
}

fn run(request: &CompileRequest, assertions: &str) {
    let artifact = codegen::compile(request).unwrap();
    assert_eq!(artifact.manifest.len(), request.regions.len());
    for (index, (manifest, region)) in artifact.manifest.iter().zip(&request.regions).enumerate() {
        assert_eq!(manifest.entry, region.ir.entry);
        assert_eq!(manifest.source, region.source);
        assert_eq!(manifest.expected_old, region.expected_old);
        assert_eq!(manifest.export, format!("region_{index}"));
    }
    let script = format!(
        r#"{}
        const data = new DataView(memory.buffer);
        function resetMemory(options) {{
            reset(options);
            new Uint8Array(memory.buffer,148,16).fill(0);
            new Uint8Array(memory.buffer,4096,4096).fill(0xa5);
            data.setUint32(A,F,true);
        }}
        {assertions}
    "#,
        include_str!("execution.js")
    );
    let mut child = Command::new("node")
        .args(["-e", &script])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin.write_all(&(artifact.bytes.len() as u32).to_le_bytes()).unwrap();
    stdin.write_all(&artifact.bytes).unwrap();
    stdin.write_all(&raw_helpers()).unwrap();
    drop(stdin);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Node generated-code execution failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn raw_helpers() -> Vec<u8> {
    let mut module = Module::new();
    let mut types = TypeSection::new();
    types.ty().function([ValType::I32; 4], [ValType::I32]);
    types.ty().function([ValType::I32; 4], []);
    module.section(&types);
    let mut functions = FunctionSection::new();
    functions.function(0).function(0).function(1);
    module.section(&functions);
    let mut memory = MemorySection::new();
    memory.memory(MemoryType {
        minimum: 1,
        maximum: None,
        memory64: false,
        shared: false,
        page_size_log2: None,
    });
    module.section(&memory);
    let mut exports = ExportSection::new();
    exports
        .export("memory", ExportKind::Memory, 0)
        .export("load", ExportKind::Func, 0)
        .export("store", ExportKind::Func, 1)
        .export("sample_prepare", ExportKind::Func, 2);
    module.section(&exports);
    let mut code = CodeSection::new();
    for store in [false, true] {
        let mut function = Function::new([(1, ValType::I32)]);
        let mut s = function.instructions();
        let mem = MemArg {
            offset: 0,
            align: 0,
            memory_index: 0,
        };
        let count = if store { 156 } else { 152 };
        s.i32_const(count).i32_const(count).i32_load(mem).i32_const(1).i32_add().i32_store(mem);
        // Test-only access adapter: guest [0x2000,0x3000) maps to host [0x1000,0x2000).
        // The control word injects the documented helper statuses, not CPU behavior.
        s.i32_const(148).i32_load(mem).i32_const(1).i32_eq().if_(wasm_encoder::BlockType::Empty);
        s.i32_const(1).return_().end();
        s.i32_const(148).i32_load(mem).i32_const(2).i32_eq().if_(wasm_encoder::BlockType::Empty);
        s.local_get(0).i32_load(mem).i32_const(84).i32_add().local_get(1).i32_store(mem);
        s.i32_const(2).return_().end();
        s.local_get(1)
            .i32_const(0x2000)
            .i32_lt_u()
            .local_get(1)
            .i32_const(0x3000)
            .i32_ge_u()
            .i32_or();
        s.local_get(1).local_get(2).i32_rem_u().i32_or().if_(wasm_encoder::BlockType::Empty);
        s.i32_const(1).return_().end();
        s.local_get(1).i32_const(0x1000).i32_sub().local_set(4);
        for width in [1, 2, 4] {
            s.local_get(2).i32_const(width).i32_eq().if_(wasm_encoder::BlockType::Empty);
            if store {
                s.local_get(4).local_get(3);
                match width {
                    1 => {
                        s.i32_store8(mem);
                    }
                    2 => {
                        s.i32_store16(mem);
                    }
                    _ => {
                        s.i32_store(mem);
                    }
                }
            } else {
                s.local_get(3).local_get(4);
                match width {
                    1 => {
                        s.i32_load8_u(mem);
                    }
                    2 => {
                        s.i32_load16_u(mem);
                    }
                    _ => {
                        s.i32_load(mem);
                    }
                }
                s.i32_store(mem);
            }
            s.end();
        }
        if store {
            s.i32_const(148).i32_load(mem);
        } else {
            s.i32_const(0);
        }
        s.end();
        code.function(&function);
    }
    let mut sample = Function::new([]);
    let mut s = sample.instructions();
    // The isolated access adapter records the sample arguments; it cannot execute guest instructions.
    for parameter in 0..4 {
        s.i32_const(128).local_get(parameter).i32_store(MemArg {
            offset: u64::from(parameter) * 4,
            align: 2,
            memory_index: 0,
        });
    }
    let count = MemArg {
        offset: 144,
        align: 2,
        memory_index: 0,
    };
    s.i32_const(0).i32_const(0).i32_load(count).i32_const(1).i32_add().i32_store(count);
    let word = MemArg {
        offset: 0,
        align: 2,
        memory_index: 0,
    };
    s.i32_const(160).i32_const(4096).i32_load(word).i32_store(word).end();
    code.function(&sample);
    module.section(&code);
    module.finish()
}

#[test]
fn guest_loop_obeys_dynamic_end_budget_and_sampling() {
    let mut branch = Instruction {
        pc: 0x1004,
        size: 2,
        condition: Condition::Ne,
        operation: Operation::Branch {
            target: Value::Immediate(0x1000),
            link: None,
            exchange: false,
        },
    };
    let mut input = request(vec![
        alu(0x1000, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false),
        alu(0x1002, AluOp::Sub, None, Value::Register(0), Value::Immediate(3), true),
        branch.clone(),
    ]);
    run(
        &input,
        r#"
        reset(); assert.equal(entry(F, A), 0); assert.equal(get(0), 3); assert.equal(get(60), 0x1006); assert.equal(get(80), 9);
        reset({end: 0x1002}); assert.equal(entry(F, A), 3); assert.equal(get(0), 1); assert.equal(get(80), 1);
        reset({end: 0x1000, budget: 0}); assert.equal(entry(F, A), 3); assert.equal(get(80), 0);
        reset({budget: 0}); assert.equal(entry(F, A), 2); assert.equal(get(80), 0);
        reset({budget: 1}); assert.equal(entry(F, A), 2); assert.equal(get(0), 1); assert.equal(get(80), 1);
        reset({sample: 2}); assert.equal(entry(F, A), 1); assert.equal(get(80), 2); assert.equal(get(76), 0);
        assert.deepEqual(samples, [[A, 0x1002, 0x30, 77]]);
        reset({end: 0x1002, budget: 1, sample: 1}); assert.equal(entry(F, A), 3); assert.equal(samples.length, 1);
        reset({pc: 8, end: 8, budget: 0}); assert.equal(entry(F, A), 6); assert.equal(get(84), 8); assert.equal(get(80), 0);
    "#,
    );
    branch.pc = 0x1000;
    branch.condition = Condition::Always;
    input.regions[0].ir.blocks[0].instructions = vec![branch];
    run(
        &input,
        "reset({budget: 10001, sample: 20000}); assert.equal(entry(F,A),2); assert.equal(get(80),10001); assert.equal(get(76),9999);",
    );
}

#[test]
fn arithmetic_flags_and_false_conditions_retire_once() {
    let cases: [(AluOp, u32, u32, u32, u32); 8] = [
        (AluOp::Add, 0xffff_ffff, 1, 0, 0x6000_0030),
        (AluOp::Add, 0x7fff_ffff, 1, 0x8000_0000, 0x9000_0030),
        (AluOp::Sub, 0, 1, 0xffff_ffff, 0x8000_0030),
        (AluOp::Sub, 0x8000_0000, 1, 0x7fff_ffff, 0x3000_0030),
        (AluOp::And, 0xf0, 0x0f, 0, 0x7000_0030),
        (AluOp::Or, 0x80, 1, 0x81, 0x3000_0030),
        (AluOp::Xor, 0x8000_0000, 1, 0x8000_0001, 0xb000_0030),
        (AluOp::Move, 99, 42, 42, 0x3000_0030),
    ];
    for (op, left, right, result, cpsr) in cases {
        let input = request(vec![alu(0x1000, op, Some(7), Value::Immediate(left), Value::Immediate(right), true)]);
        run(
            &input,
            &format!(
                "reset({{sample:1,cpsr:0x30000030}}); assert.equal(entry(F,A),1); assert.equal(get(28),{result}); assert.equal(get(64),{cpsr}); assert.equal(get(80),1); assert.deepEqual(samples,[[A,0x1000,0x30000030,77]]);"
            ),
        );
    }
    let mut skipped = alu(0x1000, AluOp::Move, Some(7), Value::Immediate(0), Value::Immediate(55), true);
    skipped.condition = Condition::Eq;
    run(
        &request(vec![skipped]),
        "reset({sample:1}); assert.equal(entry(F,A),1); assert.equal(get(28),77); assert.equal(get(64),0x30); assert.equal(get(80),1); assert.equal(get(72),99); assert.equal(samples.length,1);",
    );
}

#[test]
fn exchange_and_unsupported_operand_forms_preserve_instruction_boundaries() {
    run(
        &request(vec![Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: Operation::Branch {
                target: Value::Register(0),
                link: Some(0x1003),
                exchange: true,
            },
        }]),
        r#"
        reset(); set(0, 0x2000); assert.equal(entry(F,A),0); assert.equal(get(60),0x2000); assert.equal(get(64),0x10); assert.equal(get(56),0x1003); assert.equal(get(80),1);
        reset({sample:1}); set(0, 8); assert.equal(entry(F,A),6); assert.equal(get(80),1); assert.equal(samples.length,1); assert.equal(get(84),8);
    "#,
    );
    for condition in [Condition::Always, Condition::Eq] {
        let mut instruction = alu(0x1000, AluOp::Move, Some(15), Value::Immediate(0), Value::Register(1), true);
        instruction.condition = condition;
        run(
            &request(vec![instruction]),
            "reset({sample:1}); const before=Array.from(new Uint8Array(memory.buffer,F,92)); assert.equal(entry(F,A),4); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,92)),before); assert.equal(samples.length,0);",
        );
    }
}

#[test]
fn omitted_frontier_dispatches_without_retiring_and_keeps_the_branch_alternative() {
    run(
        &request(vec![
            Instruction {
                pc: 0x1000,
                size: 2,
                condition: Condition::Eq,
                operation: Operation::Branch {
                    target: Value::Immediate(0x1004),
                    link: None,
                    exchange: false,
                },
            },
            alu(0x1004, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false),
        ]),
        r#"
        reset(); assert.equal(entry(F,A),0); assert.equal(get(60),0x1002); assert.equal(get(80),1);
        assert.equal(get(72),99); assert.equal(get(76),99); assert.equal(get(0),0);
        reset({pc:0x1002,sample:1}); const before=Array.from(new Uint8Array(memory.buffer,F,92));
        assert.equal(entry(F,A),0); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,92)),before);
        assert.equal(samples.length,0);
        reset({cpsr:0x40000030}); assert.equal(entry(F,A),0); assert.equal(get(60),0x1006);
        assert.equal(get(80),2); assert.equal(get(0),1);
        reset({pc:0x1002,end:0x1002,budget:0,sample:0}); assert.equal(entry(F,A),3); assert.equal(get(80),0);
        reset({pc:0x1002,budget:0,sample:0}); assert.equal(entry(F,A),2); assert.equal(get(80),0);
        reset({pc:0x1002,sample:0}); assert.equal(entry(F,A),1); assert.equal(get(80),0);
    "#,
    );
}

#[test]
fn every_condition_matches_the_arm_truth_table() {
    let conditions = [
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
    let mut input = request(Vec::new());
    input.regions.clear();
    for condition in conditions {
        let mut instruction = alu(0x1000, AluOp::Move, Some(0), Value::Immediate(0), Value::Immediate(1), false);
        instruction.condition = condition;
        input.regions.extend(request(vec![instruction]).regions);
    }
    run(
        &input,
        r#"
        for (let bits=0; bits<16; bits++) {
            const n=!!(bits&8), z=!!(bits&4), c=!!(bits&2), v=!!(bits&1);
            const expected=[z,!z,c,!c,n,!n,v,!v,c&&!z,!c||z,n===v,n!==v,!z&&n===v,z||n!==v,true];
            expected.forEach((passes,index) => {
                reset({cpsr: ((bits<<28)>>>0)|0x30, budget:1});
                assert.equal(instance.exports[`region_${index}`](F,A),2);
                assert.equal(get(0),Number(passes),`condition ${index}, NZCV ${bits}`);
                assert.equal(get(80),1); assert.equal(get(76),99); assert.equal(get(60),0x1002);
            });
        }
    "#,
    );
}

#[test]
fn unsupported_forms_have_zero_effects() {
    let address = Address {
        base: Value::Register(0),
        offset: Operand {
            value: Value::Immediate(0),
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        },
        subtract: false,
        pre_index: true,
        write_back: Some(0),
    };
    let mut operations = vec![
        Operation::Load {
            destination: 15,
            address,
            width: Width::Word,
            signed: false,
        },
        Operation::Store {
            value: Value::Register(16),
            address,
            width: Width::Word,
        },
    ];
    operations.push(Operation::Alu {
        op: AluOp::Move,
        destination: Some(0),
        left: Value::Immediate(0),
        right: Operand {
            value: Value::Register(0),
            shift: Shift::Lsl,
            amount: ShiftAmount::Register(16),
        },
        set_flags: true,
    });
    operations.push(alu(0x1000, AluOp::Move, Some(15), Value::Immediate(0), Value::Register(0), false).operation);
    let mut input = request(Vec::new());
    input.regions.clear();
    for operation in operations {
        input.regions.extend(
            request(vec![Instruction {
                pc: 0x1000,
                size: 2,
                condition: Condition::Always,
                operation,
            }])
            .regions,
        );
    }
    run(
        &input,
        r#"
        for (const entry of Object.values(instance.exports)) {
            reset({sample:1}); const before=Array.from(new Uint8Array(memory.buffer,F,92));
            assert.equal(entry(F,A),4); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,92)),before); assert.equal(samples.length,0);
            reset({budget:0}); assert.equal(entry(F,A),2);
            reset({end:0x1000, budget:0}); assert.equal(entry(F,A),3);
        }
    "#,
    );
}

#[test]
fn arithmetic_matches_wide_integer_reference_with_carry_and_borrow() {
    let mut input = request(Vec::new());
    input.regions.clear();
    for thumb in [false, true] {
        for (destination, set_flags) in [(Some(0), true), (None, true), (Some(0), false)] {
            for op in [
                AluOp::Add,
                AluOp::AddCarry,
                AluOp::Sub,
                AluOp::SubCarry,
                AluOp::ReverseSub,
                AluOp::ReverseSubCarry,
            ] {
                let mut region = request(vec![alu(0x1000, op, destination, Value::Register(0), Value::Register(1), set_flags)]);
                region.regions[0].ir.entry.thumb = thumb;
                region.regions[0].ir.blocks[0].instructions[0].size = if thumb { 2 } else { 4 };
                input.regions.extend(region.regions);
            }
        }
    }
    run(
        &input,
        r#"
        const values = [0, 1, 2, 0x7ffffffe, 0x7fffffff, 0x80000000, 0x80000001, 0xfffffffe, 0xffffffff];
        let seed = 0x12345678;
        for (let i=0; i<20; i++) { seed=(Math.imul(seed,1664525)+1013904223)>>>0; values.push(seed); }
        const mask = 0xffffffffn;
        for (let mode=0; mode<2; mode++) for (let variant=0; variant<3; variant++) for (let op=0; op<6; op++) {
            const execute = instance.exports[`region_${mode*18+variant*6+op}`];
            for (const left of values) for (const right of values) for (let c=0; c<2; c++) {
                const old = (0xd8000010 | (mode*0x20) | (c<<29))>>>0;
                reset({cpsr:old,budget:1}); set(0,left); set(4,right);
                let a=BigInt(left), b=BigInt(right);
                if (op>=4) [a,b]=[b,a];
                const carry = op%2 ? BigInt(c) : 0n;
                const exact = op<2 ? a+b+carry : a-b-(op%2 ? 1n-carry : 0n);
                const signed = op<2 ? BigInt.asIntN(32,a)+BigInt.asIntN(32,b)+carry
                    : BigInt.asIntN(32,a)-BigInt.asIntN(32,b)-(op%2 ? 1n-carry : 0n);
                const result = Number(exact & mask);
                const overflow = signed < -0x80000000n || signed > 0x7fffffffn;
                const carryOut = op<2 ? exact>mask : exact>=0n;
                const flags = ((old&0x0fffffff) | (result&0x80000000) | (Number(result===0)<<30)
                    | (Number(carryOut)<<29) | (Number(overflow)<<28))>>>0;
                assert.equal(execute(F,A),2);
                assert.equal(get(0),variant===1 ? left : result,`op ${op}, ${left}, ${right}, C=${c}`);
                assert.equal(get(4),right); assert.equal(get(64),variant===2 ? old : flags);
                assert.equal(get(60),mode ? 0x1002 : 0x1004); assert.equal(get(80),1);
                assert.equal(get(72),0); assert.equal(get(76),99);
            }
        }
    "#,
    );
}

#[test]
fn bit_clear_not_and_multiply_preserve_carry_and_overflow() {
    let mut input = request(Vec::new());
    input.regions.clear();
    for set_flags in [true, false] {
        for op in [AluOp::BitClear, AluOp::Not, AluOp::Multiply] {
            input
                .regions
                .extend(request(vec![alu(0x1000, op, Some(1), Value::Register(0), Value::Register(1), set_flags)]).regions);
        }
    }
    run(
        &input,
        r#"
        const values = [0, 1, 2, 0x80000000, 0x7fffffff, 0xffffffff, 0x12345678];
        for (let variant=0; variant<2; variant++) for (let op=0; op<3; op++) {
            const execute=instance.exports[`region_${variant*3+op}`];
            for (const left of values) for (const right of values) for (let cv=0; cv<4; cv++) {
                const old=(0xc8000030 | (cv<<28))>>>0;
                const result=op===0 ? (left&~right)>>>0 : op===1 ? (~right)>>>0
                    : Number((BigInt(left)*BigInt(right))&0xffffffffn);
                reset({cpsr:old,sample:1}); set(0,left); set(4,right);
                assert.equal(execute(F,A),1); assert.equal(get(4),result); assert.equal(get(0),left);
                assert.equal(get(64),variant ? old : ((old&0x3fffffff) | (result&0x80000000) | (Number(result===0)<<30))>>>0);
                assert.equal(get(80),1); assert.deepEqual(samples,[[A,0x1000,old,77]]);
            }
        }
    "#,
    );
}

#[test]
fn barrel_shifts_match_bit_precision_reference_at_word_boundaries() {
    let mut operands = Vec::new();
    for shift in [Shift::Lsl, Shift::Lsr, Shift::Asr, Shift::Ror] {
        operands.push(Operand {
            value: Value::Register(1),
            shift,
            amount: ShiftAmount::Register(2),
        });
        for amount in [0, 1, 2, 31, 32, 33, 255] {
            for value in [
                Value::Register(1),
                Value::Immediate(2),
                Value::Immediate(0x81),
                Value::Immediate(0x8000_0001),
            ] {
                operands.push(Operand {
                    value,
                    shift,
                    amount: ShiftAmount::Immediate(amount),
                });
            }
        }
    }
    for value in [Value::Register(1), Value::Immediate(2), Value::Immediate(3)] {
        operands.push(Operand {
            value,
            shift: Shift::Rrx,
            amount: ShiftAmount::Immediate(1),
        });
    }
    let mut input = request(Vec::new());
    input.regions.clear();
    for right in &operands {
        input.regions.extend(
            request(vec![Instruction {
                pc: 0x1000,
                size: 2,
                condition: Condition::Always,
                operation: Operation::Alu {
                    op: AluOp::Move,
                    destination: Some(1),
                    left: Value::Immediate(0),
                    right: *right,
                    set_flags: true,
                },
            }])
            .regions,
        );
    }
    run(
        &input,
        &format!(
            r#"
        const operands = {};
        const values = [0, 1, 2, 0x7fffffff, 0x80000000, 0x80000001, 0xffffffff, 0x12345678];
        for (let i=0; i<operands.length; i++) for (const value of values)
            for (const count of [0,1,2,31,32,33,63,64,255,256,257,511,0xffffffff]) for (let c=0; c<2; c++) {{
                const operand=operands[i], shift=operand.shift;
                const n=BigInt(operand.amount.Immediate ?? (count&255));
                const x=BigInt(operand.value.Immediate ?? value);
                let result=x, carry=c;
                if (shift==='Rrx') {{ result=(BigInt(c)<<31n)|(x>>1n); carry=Number(x&1n); }}
                else if (n) {{
                    if (shift==='Lsl') {{ const wide=x<<n; result=wide&0xffffffffn; carry=Number((wide>>32n)&1n); }}
                    if (shift==='Lsr') {{ result=x>>n; carry=Number((x>>(n-1n))&1n); }}
                    if (shift==='Asr') {{ result=BigInt.asIntN(32,x)>>n; carry=Number((BigInt.asIntN(32,x)>>(n-1n))&1n); }}
                    if (shift==='Ror') {{ const r=n%32n; result=((x>>r)|(x<<(32n-r)))&0xffffffffn; carry=Number(result>>31n); }}
                }}
                result=Number(BigInt.asUintN(32,result));
                const old=(0xd8000030|(c<<29))>>>0;
                reset({{cpsr:old,budget:1}}); set(4,value); set(8,count);
                assert.equal(instance.exports[`region_${{i}}`](F,A),2);
                assert.equal(get(4),result,`${{shift}} ${{value}} by ${{n}}, C=${{c}}`);
                assert.equal(get(64),((old&0x1fffffff)|(result&0x80000000)|(Number(result===0)<<30)|(carry<<29))>>>0);
                assert.equal(get(8),count); assert.equal(get(80),1);
            }}
    "#,
            serde_json::to_string(&operands).unwrap()
        ),
    );
}

#[test]
fn constant_alu_keeps_dynamic_carry_conditions_and_retirement_boundaries() {
    let mut input = request(Vec::new());
    input.regions.clear();
    let mut operations = Vec::new();
    for op in [
        AluOp::Add,
        AluOp::Sub,
        AluOp::ReverseSub,
        AluOp::And,
        AluOp::Or,
        AluOp::Xor,
        AluOp::BitClear,
        AluOp::Not,
        AluOp::Multiply,
        AluOp::Move,
        AluOp::AddCarry,
        AluOp::SubCarry,
        AluOp::ReverseSubCarry,
    ] {
        for (value, shift, amount) in [
            (4, Shift::Lsl, 0),
            (2, Shift::Lsl, 0),
            (2, Shift::Ror, 2),
            (4, Shift::Ror, 2),
            (3, Shift::Rrx, 1),
        ] {
            let operation = Operation::Alu {
                op,
                destination: Some(0),
                left: Value::Immediate(0x1008),
                right: Operand {
                    value: Value::Immediate(value),
                    shift,
                    amount: ShiftAmount::Immediate(amount),
                },
                set_flags: op == AluOp::Move,
            };
            operations.push(operation.clone());
            let mut region = request(vec![Instruction {
                pc: 0x1000,
                size: 4,
                condition: Condition::Ne,
                operation,
            }]);
            region.regions[0].ir.entry.thumb = false;
            input.regions.extend(region.regions);
        }
    }
    run(
        &input,
        &format!(
            r#"
        const operations = {};
        operations.forEach(({{Alu:op}}, i) => {{
            for (let c=0; c<2; c++) for (const passes of [false,true])
                for (const budget of [0,1,2]) for (const sample of [0,1,2]) for (const end of [0x1000,0x1004,0xfffffff0]) {{
                    const old=(0x18000010|(c<<29)|(Number(!passes)<<30))>>>0;
                    const a=BigInt(op.left.Immediate), x=BigInt(op.right.value.Immediate), n=BigInt(op.right.amount.Immediate);
                    let b=x, carry=c;
                    if (op.right.shift==='Ror') {{ b=((x>>n)|(x<<(32n-n)))&0xffffffffn; carry=Number(b>>31n); }}
                    if (op.right.shift==='Rrx') {{ b=(BigInt(c)<<31n)|(x>>1n); carry=Number(x&1n); }}
                    const exact={{Add:a+b,Sub:a-b,ReverseSub:b-a,And:a&b,Or:a|b,Xor:a^b,BitClear:a&~b,
                        Not:~b,Multiply:a*b,Move:b,AddCarry:a+b+BigInt(c),SubCarry:a-b-1n+BigInt(c),ReverseSubCarry:b-a-1n+BigInt(c)}}[op.op];
                    const result=Number(BigInt.asUintN(32,exact));
                    const retired=Number(end!==0x1000 && budget!==0 && sample!==0), changed=retired && passes;
                    const flags=changed && op.set_flags ? ((old&0x1fffffff)|(result&0x80000000)|(Number(result===0)<<30)|(carry<<29))>>>0 : old;
                    const exit=end===0x1000 ? 3 : budget===0 ? 2 : sample===0 ? 1 : end===0x1004 ? 3 : budget===1 ? 2 : sample===1 ? 1 : 0;
                    reset({{cpsr:old,budget,sample,end}}); set(0,0xdeadbeef);
                    assert.equal(instance.exports[`region_${{i}}`](F,A),exit);
                    assert.equal(get(0),changed ? result : 0xdeadbeef,`${{op.op}}, C=${{c}}`);
                    assert.equal(get(64),flags); assert.equal(get(60),0x1000+4*retired); assert.equal(get(80),retired);
                    assert.equal(get(72),budget-retired); assert.equal(get(76),sample-retired);
                    assert.deepEqual(samples,retired && sample===1 ? [[A,0x1000,old,77]] : []);
                }}
        }});
    "#,
            serde_json::to_string(&operations).unwrap()
        ),
    );
}

#[test]
fn shifted_operands_use_shifter_carry_only_for_logical_flags() {
    let ops = [
        AluOp::And,
        AluOp::Xor,
        AluOp::Or,
        AluOp::BitClear,
        AluOp::Not,
        AluOp::AddCarry,
        AluOp::SubCarry,
        AluOp::ReverseSubCarry,
    ];
    let mut input = request(Vec::new());
    input.regions.clear();
    for set_flags in [true, false] {
        for op in ops {
            let mut instruction = alu(0x1000, op, Some(0), Value::Register(0), Value::Register(1), set_flags);
            if let Operation::Alu { right, .. } = &mut instruction.operation {
                right.amount = ShiftAmount::Immediate(1);
            }
            input.regions.extend(request(vec![instruction]).regions);
        }
    }
    run(
        &input,
        r#"
        for (let variant=0; variant<2; variant++) for (let op=0; op<8; op++)
            for (const a of [0,1,0x7fffffff,0x80000000,0xffffffff])
            for (const b of [0,1,0x40000000,0x80000000,0xffffffff]) for (let c=0; c<2; c++) {
                const old=(0xd8000030|(c<<29))>>>0, shifted=(b<<1)>>>0;
                let result, carry=b>>>31, overflow=1;
                if (op<5) result=[a&shifted,a^shifted,a|shifted,a&~shifted,~shifted][op]>>>0;
                else {
                    const x=BigInt(a), y=BigInt(shifted), sx=BigInt.asIntN(32,x), sy=BigInt.asIntN(32,y);
                    const exact=op===5 ? x+y+BigInt(c) : op===6 ? x-y-1n+BigInt(c) : y-x-1n+BigInt(c);
                    const signed=op===5 ? sx+sy+BigInt(c) : op===6 ? sx-sy-1n+BigInt(c) : sy-sx-1n+BigInt(c);
                    result=Number(BigInt.asUintN(32,exact)); carry=Number(op===5 ? exact>0xffffffffn : exact>=0n);
                    overflow=Number(signed < -0x80000000n || signed > 0x7fffffffn);
                }
                reset({cpsr:old,budget:1}); set(0,a); set(4,b);
                assert.equal(instance.exports[`region_${variant*8+op}`](F,A),2); assert.equal(get(0),result);
                assert.equal(get(64),variant ? old : ((old&0x0fffffff)|(result&0x80000000)|(Number(result===0)<<30)|(carry<<29)|(overflow<<28))>>>0);
            }
    "#,
    );
}

#[test]
fn single_memory_accesses_apply_width_sign_indexing_and_writeback() {
    let mut operations = Vec::new();
    for width in [Width::Byte, Width::Half, Width::Word] {
        for (pre_index, write_back) in [(true, None), (true, Some(0)), (false, Some(0))] {
            for subtract in [false, true] {
                for offset in [
                    Operand {
                        value: Value::Immediate(8),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    Operand {
                        value: Value::Register(2),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(2),
                    },
                ] {
                    let address = Address {
                        base: Value::Register(0),
                        offset,
                        subtract,
                        pre_index,
                        write_back,
                    };
                    for signed in [false, true] {
                        if signed && width == Width::Word {
                            continue;
                        }
                        operations.push(Operation::Load {
                            destination: 7,
                            address,
                            width,
                            signed,
                        });
                    }
                    operations.push(Operation::Store {
                        value: Value::Register(1),
                        address,
                        width,
                    });
                }
            }
        }
    }
    let mut input = request(Vec::new());
    input.regions.clear();
    for operation in &operations {
        let mut region = request(vec![Instruction {
            pc: 0x1000,
            size: 4,
            condition: Condition::Always,
            operation: operation.clone(),
        }]);
        region.regions[0].ir.entry.thumb = false;
        input.regions.extend(region.regions);
    }
    run(
        &input,
        &format!(
            r#"
        const operations={};
        operations.forEach((operation,i) => {{ for (const initial of [0,0x7f,0x80,0x7fff,0x8000,0xffffffff,0x8012ff80]) {{
            const load=!!operation.Load, op=operation.Load ?? operation.Store, addr=op.address;
            const width={{Byte:1,Half:2,Word:4}}[op.width];
            const updated=0x2010+(addr.subtract ? -8 : 8), effective=addr.pre_index ? updated : 0x2010;
            resetMemory({{cpsr:0xf8000010,sample:1}}); set(0,0x2010); set(4,0x89abcdef); set(8,2);
            data.setUint32(effective-0x1000,initial,true);
            const expected=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
            const view=new DataView(expected.buffer), offset=effective-0x2000;
            let result=77;
            if (load) result=(width===1 ? (op.signed ? view.getInt8(offset) : view.getUint8(offset))
                : width===2 ? (op.signed ? view.getInt16(offset,true) : view.getUint16(offset,true))
                : view.getUint32(offset,true))>>>0;
            else if (width===1) view.setUint8(offset,0x89abcdef);
            else if (width===2) view.setUint16(offset,0x89abcdef,true);
            else view.setUint32(offset,0x89abcdef,true);
            assert.equal(instance.exports[`region_${{i}}`](F,A),1);
            assert.equal(get(0),addr.write_back===null ? 0x2010 : updated); assert.equal(get(28),result);
            assert.equal(get(4),0x89abcdef); assert.equal(get(8),2); assert.equal(get(64),0xf8000010);
            assert.equal(get(60),0x1004); assert.equal(get(80),1); assert.equal(get(72),99); assert.equal(get(76),0);
            assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),expected);
            assert.equal(data.getUint32(152,true),Number(load)); assert.equal(data.getUint32(156,true),Number(!load));
            assert.deepEqual(samples,[[A,0x1000,0xf8000010,77]]);
        }} }});
    "#,
            serde_json::to_string(&operations).unwrap()
        ),
    );
}

#[test]
fn memory_fallback_and_fault_do_not_commit_the_failed_instruction() {
    let address = Address {
        base: Value::Register(0),
        offset: Operand {
            value: Value::Immediate(4),
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        },
        subtract: false,
        pre_index: false,
        write_back: Some(0),
    };
    for operation in [
        Operation::Load {
            destination: 7,
            address,
            width: Width::Word,
            signed: false,
        },
        Operation::Store {
            value: Value::Register(1),
            address,
            width: Width::Word,
        },
    ] {
        run(
            &request(vec![
                alu(0x1000, AluOp::Add, Some(3), Value::Register(3), Value::Immediate(1), false),
                Instruction {
                    pc: 0x1002,
                    size: 2,
                    condition: Condition::Always,
                    operation,
                },
            ]),
            r#"
            for (const [address,status,exit] of [[0x2001,0,4],[0x3000,0,4],[0xfffffffc,0,4],[0x2000,1,4],[0x2000,2,6]]) {
                resetMemory({sample:2}); set(0,address); set(4,0x12345678); set(88,0xdeadbeef);
                data.setUint32(148,status,true);
                const before=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
                assert.equal(entry(F,A),exit); assert.equal(get(0),address); assert.equal(get(28),77);
                assert.equal(get(4),0x12345678); assert.equal(get(12),1); assert.equal(get(64),0x30);
                assert.equal(get(60),0x1002); assert.equal(get(80),1); assert.equal(get(72),99); assert.equal(get(76),1);
                assert.equal(get(84),status===2 ? address : 0); assert.equal(get(88),0xdeadbeef);
                assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),before);
                // This is a staged sample only: the failed instruction has not retired.
                assert.deepEqual(samples,[[A,0x1002,0x30,77]]);
            }
        "#,
        );
    }
}

#[test]
fn invalidating_store_retires_before_exit_with_normal_boundary_priority() {
    let mut store = Instruction {
        pc: 0x1000,
        size: 2,
        condition: Condition::Always,
        operation: Operation::Store {
            value: Value::Register(1),
            width: Width::Word,
            address: Address {
                base: Value::Register(0),
                offset: Operand {
                    value: Value::Immediate(4),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0),
                },
                subtract: false,
                pre_index: false,
                write_back: Some(0),
            },
        },
    };
    run(
        &request(vec![
            store.clone(),
            alu(0x1002, AluOp::Add, Some(3), Value::Register(3), Value::Immediate(1), false),
        ]),
        r#"
        for (const [options,exit] of [[{},5],[{sample:1},1],[{budget:1,sample:1},2],[{end:0x1002,budget:1,sample:1},3]]) {
            resetMemory(options); set(0,0x2000); set(4,0x12345678); set(28,0x2000); data.setUint32(148,3,true);
            assert.equal(entry(F,A),exit); assert.equal(data.getUint32(4096,true),0x12345678);
            assert.equal(get(0),0x2004); assert.equal(get(12),0); assert.equal(get(60),0x1002); assert.equal(get(80),1);
            assert.equal(get(72),(options.budget??100)-1); assert.equal(get(76),(options.sample??100)-1);
            assert.equal(data.getUint32(156,true),1);
            if (options.sample===1) { assert.equal(data.getUint32(160,true),0xa5a5a5a5); assert.deepEqual(samples,[[A,0x1000,0x30,0x2000]]); }
            else assert.equal(samples.length,0);
        }
    "#,
    );
    store.pc = 0xffff_fffe;
    let mut input = request(vec![store.clone()]);
    input.regions[0].ir.entry.pc = store.pc;
    run(
        &input,
        r#"
        resetMemory({pc:0xfffffffe,end:0,budget:1,sample:1}); set(0,0x2000); set(4,42); data.setUint32(148,3,true);
        assert.equal(entry(F,A),6); assert.equal(get(84),0); assert.equal(get(60),0); assert.equal(get(80),1);
        assert.equal(get(72),0); assert.equal(get(76),0); assert.equal(get(0),0x2004); assert.equal(data.getUint32(4096,true),42);
        assert.equal(samples.length,1);
    "#,
    );
    store.pc = 0x1000;
    store.condition = Condition::Eq;
    run(
        &request(vec![
            store,
            alu(0x1002, AluOp::Add, Some(3), Value::Register(3), Value::Immediate(1), false),
        ]),
        r#"
        resetMemory(); set(0,0x2001); data.setUint32(148,3,true);
        assert.equal(entry(F,A),0); assert.equal(get(0),0x2001); assert.equal(get(12),1); assert.equal(get(80),2);
        assert.equal(data.getUint32(156,true),0); assert.equal(data.getUint32(4096,true),0xa5a5a5a5);
    "#,
    );
}

#[test]
fn memory_addresses_snapshot_aliased_operands_and_wrap_to_32_bits() {
    let mut instruction = Instruction {
        pc: 0x1000,
        size: 2,
        condition: Condition::Always,
        operation: Operation::Load {
            destination: 1,
            width: Width::Word,
            signed: false,
            address: Address {
                base: Value::Register(0),
                offset: Operand {
                    value: Value::Register(1),
                    shift: Shift::Rrx,
                    amount: ShiftAmount::Immediate(1),
                },
                subtract: false,
                pre_index: true,
                write_back: Some(0),
            },
        },
    };
    run(
        &request(vec![instruction.clone()]),
        r#"
        resetMemory({cpsr:0x30000030,budget:1}); set(0,0x80002000); set(4,8); data.setUint32(4100,0x89abcdef,true);
        assert.equal(entry(F,A),2); assert.equal(get(0),0x2004); assert.equal(get(4),0x89abcdef); assert.equal(get(64),0x30000030);
        assert.equal(get(88),0x89abcdef); assert.equal(get(80),1);
    "#,
    );
    if let Operation::Load { destination, address, .. } = &mut instruction.operation {
        *destination = 0;
        address.offset = Operand {
            value: Value::Immediate(4),
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        };
        address.subtract = true;
        address.write_back = None;
    }
    run(
        &request(vec![instruction.clone()]),
        r#"
        resetMemory({budget:1}); set(0,0x2004); data.setUint32(4096,0xdeadbeef,true);
        assert.equal(entry(F,A),2); assert.equal(get(0),0xdeadbeef); assert.equal(get(80),1);
    "#,
    );
    if let Operation::Load { address, .. } = &mut instruction.operation {
        address.base = Value::Register(15);
        address.offset.value = Value::Immediate(0xff8);
        address.subtract = false;
    }
    instruction.size = 4;
    let mut input = request(vec![instruction.clone()]);
    input.regions[0].ir.entry.thumb = false;
    run(
        &input,
        r#"
        resetMemory({cpsr:0x10,budget:1}); data.setUint32(4096,0xcafebabe,true);
        assert.equal(entry(F,A),2); assert.equal(get(0),0xcafebabe); assert.equal(get(60),0x1004);
    "#,
    );
    instruction.size = 2;
    instruction.condition = Condition::Eq;
    run(
        &request(vec![instruction]),
        r#"
        resetMemory({sample:1}); data.setUint32(148,2,true);
        assert.equal(entry(F,A),1); assert.equal(get(0),0); assert.equal(get(80),1);
        assert.equal(data.getUint32(152,true),0); assert.equal(get(84),0); assert.equal(samples.length,1);
    "#,
    );
}

#[test]
fn byte_copy_loop_keeps_memory_and_retirement_across_sample_resume() {
    let mut instructions = Vec::new();
    for (pc, base) in [(0x1000, 0), (0x1004, 1)] {
        let address = Address {
            base: Value::Register(base),
            offset: Operand {
                value: Value::Immediate(1),
                shift: Shift::Lsl,
                amount: ShiftAmount::Immediate(0),
            },
            subtract: false,
            pre_index: false,
            write_back: Some(base),
        };
        instructions.push(Instruction {
            pc,
            size: 4,
            condition: Condition::Always,
            operation: if base == 0 {
                Operation::Load {
                    destination: 2,
                    address,
                    width: Width::Byte,
                    signed: false,
                }
            } else {
                Operation::Store {
                    value: Value::Register(2),
                    address,
                    width: Width::Byte,
                }
            },
        });
    }
    let mut subtract = alu(0x1008, AluOp::Sub, Some(3), Value::Register(3), Value::Immediate(1), true);
    subtract.size = 4;
    instructions.push(subtract);
    instructions.push(Instruction {
        pc: 0x100c,
        size: 4,
        condition: Condition::Ne,
        operation: Operation::Branch {
            target: Value::Immediate(0x1000),
            link: None,
            exchange: false,
        },
    });
    let mut input = request(instructions);
    input.regions[0].ir.entry.thumb = false;
    run(
        &input,
        r#"
        for (const sampled of [false,true]) {
            resetMemory({cpsr:0x10,budget:200,sample:sampled ? 5 : 200});
            set(0,0x2000); set(4,0x2100); set(12,16);
            const source=Uint8Array.from({length:16},(_,i)=>(i*19)&255);
            new Uint8Array(memory.buffer,4096,16).set(source);
            let executed=0;
            if (sampled) {
                assert.equal(entry(F,A),1); assert.equal(get(80),5); executed=5;
                assert.equal(get(0),0x2002); assert.equal(get(4),0x2101); assert.equal(get(60),0x1004);
                assert.deepEqual(samples,[[A,0x1000,0x20000010,77]]);
                set(76,200); set(80,0); new Uint8Array(memory.buffer,128,20).fill(0);
            }
            assert.equal(entry(F,A),0); assert.equal(executed+get(80),64);
            assert.equal(get(0),0x2010); assert.equal(get(4),0x2110); assert.equal(get(12),0); assert.equal(get(60),0x1010);
            assert.equal(get(72),136); assert.equal(get(64),0x60000010);
            assert.deepEqual(new Uint8Array(memory.buffer,4352,16),source);
            assert.deepEqual(new Uint8Array(memory.buffer,4096,16),source);
            assert.equal(data.getUint32(152,true),16); assert.equal(data.getUint32(156,true),16);
        }
    "#,
    );
}

#[test]
fn arm_pc_reads_and_exchange_do_not_execute_the_other_instruction_set() {
    let mut input = request(vec![alu(0x1000, AluOp::Move, Some(0), Value::Immediate(0), Value::Register(15), false)]);
    input.regions[0].ir.entry.thumb = false;
    input.regions[0].ir.blocks[0].instructions[0].size = 4;
    run(
        &input,
        "reset({cpsr:0x10,budget:1}); assert.equal(entry(F,A),2); assert.equal(get(0),0x1008); assert.equal(get(60),0x1004); reset(); assert.equal(entry(F,A),0); assert.equal(get(80),0);",
    );
    input.regions[0].ir.blocks[0].instructions[0].operation = Operation::Branch {
        target: Value::Register(14),
        link: Some(0x1004),
        exchange: true,
    };
    run(
        &input,
        "reset({cpsr:0x10}); set(56,0x1001); assert.equal(entry(F,A),0); assert.equal(get(60),0x1000); assert.equal(get(64),0x30); assert.equal(get(56),0x1004); assert.equal(get(80),1);",
    );
}

#[test]
fn taken_branch_checks_dynamic_end_before_target_instruction() {
    let mut input = request(vec![Instruction {
        pc: 0x1000,
        size: 2,
        condition: Condition::Always,
        operation: Operation::Branch {
            target: Value::Immediate(0x1004),
            link: None,
            exchange: false,
        },
    }]);
    input.regions[0].ir.blocks.push(BasicBlock {
        instructions: vec![alu(0x1004, AluOp::Move, Some(7), Value::Immediate(0), Value::Immediate(99), false)],
    });
    run(
        &input,
        r#"
        reset({end:0x1004,budget:1,sample:1}); assert.equal(entry(F,A),3);
        assert.equal(get(60),0x1004); assert.equal(get(28),77); assert.equal(get(80),1);
        assert.deepEqual(samples,[[A,0x1000,0x30,77]]);
        reset(); assert.equal(entry(F,A),0); assert.equal(get(28),99); assert.equal(get(80),2);
    "#,
    );
}

#[test]
fn malformed_instruction_boundaries_are_compile_errors() {
    let instruction = alu(0x1000, AluOp::Move, Some(0), Value::Immediate(0), Value::Immediate(1), false);
    for size in [0, 1, 3, 4] {
        let mut malformed = instruction.clone();
        malformed.size = size;
        assert!(codegen::compile(&request(vec![malformed])).is_err());
    }
    assert!(codegen::compile(&request(vec![instruction.clone(), instruction])).is_err());
    assert!(codegen::compile(&request(Vec::new())).is_err());
    assert!(
        codegen::compile(&request(vec![
            alu(0x1000, AluOp::Move, Some(0), Value::Immediate(0), Value::Immediate(1), false),
            alu(0xffff_fffe, AluOp::Move, Some(0), Value::Immediate(0), Value::Immediate(2), false),
        ]))
        .is_err()
    );
}

#[test]
fn sparse_region_dispatch_accepts_exact_interior_pcs_and_preserves_boundaries() {
    let mut instructions: Vec<_> = (0..128)
        .map(|index| alu(0x2000 + index * 2, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false))
        .collect();
    instructions.insert(
        0,
        Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: Operation::Branch {
                target: Value::Immediate(0x2000),
                link: None,
                exchange: false,
            },
        },
    );
    run(
        &request(instructions),
        r#"
        reset({budget:200,sample:200}); assert.equal(entry(F,A),0);
        assert.equal(get(0),128); assert.equal(get(60),0x2100); assert.equal(get(80),129);
        for (let offset=0; offset<128; offset++) {
            reset({pc:0x2000+offset*2,budget:200,sample:200}); assert.equal(entry(F,A),0);
            assert.equal(get(0),128-offset); assert.equal(get(80),128-offset);
            reset({pc:0x2000+offset*2,budget:1,sample:1,end:0x2002+offset*2});
            assert.equal(entry(F,A),3); assert.equal(get(0),1); assert.equal(get(80),1);
            assert.deepEqual(samples,[[A,0x2000+offset*2,0x30,77]]);
        }
        for (const pc of [0x1001,0x1002,0x1ffe,0x2001,0x2100,0x4000,0xfffffffe]) {
            reset({pc}); const before=Array.from(new Uint8Array(memory.buffer,F,92));
            assert.equal(entry(F,A),0); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,92)),before);
        }
    "#,
    );
}
