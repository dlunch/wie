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
            new Uint8Array(memory.buffer,148,44).fill(0);
            new Uint8Array(memory.buffer,4096,4096).fill(0xa5);
            new Uint8Array(memory.buffer,16384,64).fill(0xa5);
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
    types.ty().function([ValType::I32; 3], [ValType::I64]);
    module.section(&types);
    let mut functions = FunctionSection::new();
    functions.function(0).function(0).function(1).function(2);
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
        .export("sample_prepare", ExportKind::Func, 2)
        .export("word_range", ExportKind::Func, 3);
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
        if !store {
            s.i32_const(148).i32_load(mem).i32_const(3).i32_eq().if_(wasm_encoder::BlockType::Empty);
            s.unreachable().end();
        }
        // Test-only access adapter: guest [0x2000,0x3000) maps to host [0x1000,0x2000).
        // Status controls model helper results; the load-only trap above is a backend failure.
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
        s.i32_const(176)
            .i32_load(mem)
            .i32_const(16)
            .i32_mul()
            .i32_const(8192)
            .i32_add()
            .local_set(4);
        s.local_get(4).i32_const(i32::from(store)).i32_store(mem);
        for parameter in 1..=2 {
            s.local_get(4).local_get(parameter).i32_store(MemArg {
                offset: u64::from(parameter) * 4,
                ..mem
            });
        }
        s.local_get(4).local_get(3);
        if !store {
            s.i32_load(mem);
        }
        s.i32_store(MemArg { offset: 12, ..mem });
        s.i32_const(176).i32_const(176).i32_load(mem).i32_const(1).i32_add().i32_store(mem);
        s.i32_const(0);
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
    s.i32_const(160).i32_const(4096).i32_load(word).i32_store(word);
    s.i32_const(188).i32_const(256 + 92).i32_load(word).i32_store(word).end();
    code.function(&sample);
    let mut query = Function::new([]);
    let mut s = query.instructions();
    s.i32_const(164).i32_const(164).i32_load(word).i32_const(1).i32_add().i32_store(word);
    s.i32_const(168).local_get(1).i32_store(word);
    s.i32_const(172).local_get(2).i32_store(word);
    s.i32_const(180).local_get(0).i32_store(word);
    // Admission observes only mapping/alignment and the test adapter's decline control.
    s.i32_const(148).i32_load(word).i32_eqz();
    s.local_get(1).i32_const(3).i32_and().i32_eqz().i32_and();
    s.local_get(1).i32_const(0x2000).i32_ge_u().i32_and();
    s.local_get(1).i32_const(0x3000).i32_lt_u().i32_and();
    s.local_get(1)
        .local_get(2)
        .i32_const(4)
        .i32_mul()
        .i32_add()
        .i32_const(0x3000)
        .i32_le_u()
        .i32_and()
        .if_(wasm_encoder::BlockType::Result(ValType::I64));
    s.local_get(0)
        .i32_load(word)
        .local_get(2)
        .i32_const(4)
        .i32_mul()
        .i32_store(MemArg { offset: 88, ..word });
    // A test-only split maps the suffix elsewhere in host memory, not next to the prefix.
    s.i32_const(184)
        .i32_load(word)
        .i32_eqz()
        .if_(wasm_encoder::BlockType::Result(ValType::I64));
    s.i64_const(0).else_();
    s.local_get(0)
        .i32_load(word)
        .i32_const(184)
        .i32_load(word)
        .i32_store(MemArg { offset: 88, ..word });
    s.i64_const(0x4000_i64 << 32).end();
    s.local_get(1).i32_const(0x1000).i32_sub().i64_extend_i32_u().i64_or();
    s.else_().i64_const(0).end().end();
    code.function(&query);
    module.section(&code);
    module.finish()
}

#[test]
fn alu_pc_writes_commit_dynamic_and_constant_targets_in_the_same_state() {
    for thumb in [false, true] {
        for constant in [false, true] {
            let mut instruction = alu(
                0x1000,
                AluOp::Move,
                Some(15),
                Value::Immediate(0),
                if constant { Value::Immediate(0x2003) } else { Value::Register(0) },
                false,
            );
            instruction.size = if thumb { 2 } else { 4 };
            let mut input = request(vec![instruction]);
            input.regions[0].ir.entry.thumb = thumb;
            run(
                &input,
                &format!(
                    "reset({{cpsr:{},budget:1,sample:1}}); set(0,0x2003); assert.equal(entry(F,A),2); assert.equal(get(60),{}); assert.equal(get(64),{}); assert.equal(get(80),1); assert.equal(samples.length,1);",
                    if thumb { 0xf8000030_u32 } else { 0xf8000010 },
                    if thumb { 0x2002 } else { 0x2000 },
                    if thumb { 0xf8000030_u32 } else { 0xf8000010 },
                ),
            );
        }
    }
    for thumb in [false, true] {
        let mut jump = alu(0x1000, AluOp::Add, Some(15), Value::Register(0), Value::Immediate(3), false);
        jump.size = if thumb { 2 } else { 4 };
        let mut destination = alu(0x1004, AluOp::Move, Some(7), Value::Immediate(0), Value::Immediate(99), false);
        destination.size = jump.size;
        let mut input = request(vec![jump]);
        input.regions[0].ir.blocks.push(BasicBlock {
            instructions: vec![destination],
        });
        input.regions[0].ir.entry.thumb = thumb;
        run(
            &input,
            &format!(
                r#"
            reset({{cpsr:{cpsr},budget:2,sample:2}}); set(0,0x1002);
            assert.equal(entry(F,A),2); assert.equal(get(28),99); assert.equal(get(80),2); assert.equal(get(60),{end});
            assert.equal(get(64),{cpsr}); assert.deepEqual(samples,[[A,0x1004,{cpsr},77]]);
        "#,
                cpsr = if thumb { 0x30 } else { 0x10 },
                end = if thumb { 0x1006 } else { 0x1008 }
            ),
        );
    }
}

#[test]
fn multiply_accumulate_long_and_clz_match_independent_integer_oracles() {
    let mut operations = Vec::new();
    for set_flags in [false, true] {
        for destination in 0..4 {
            operations.push(Operation::MultiplyAccumulate {
                destination,
                left: 0,
                right: 1,
                accumulate: 2,
                set_flags,
            });
        }
        for signed in [false, true] {
            for accumulate in [false, true] {
                for (low, high) in [(2, 3), (0, 1), (1, 0), (0, 3)] {
                    operations.push(Operation::MultiplyLong {
                        low,
                        high,
                        left: 0,
                        right: 1,
                        signed,
                        accumulate,
                        set_flags,
                    });
                }
            }
        }
        operations.push(alu(0x1000, AluOp::Multiply, Some(0), Value::Register(0), Value::Register(1), set_flags).operation);
    }
    for destination in [0, 1] {
        operations.push(
            alu(
                0x1000,
                AluOp::CountLeadingZeros,
                Some(destination),
                Value::Immediate(0),
                Value::Register(0),
                false,
            )
            .operation,
        );
    }
    for value in [0, 1, 0x8000_0000, u32::MAX] {
        operations.push(
            alu(
                0x1000,
                AluOp::CountLeadingZeros,
                Some(0),
                Value::Immediate(0),
                Value::Immediate(value),
                false,
            )
            .operation,
        );
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
        const vectors=[
            [0,0,0,0], [0xffffffff,0xffffffff,0xffffffff,0xffffffff],
            [0x80000000,2,0,0], [0x7fffffff,0x7fffffff,1,0x80000000],
            [0x80000000,0xffffffff,0x80000000,0xffffffff], [1,1,0xffffffff,0xffffffff],
            [0x10000,0x10000,0,0], [0x12345678,0x87654321,0xdeadbeef,0xcafebabe],
        ];
        let seed=0x12345678;
        for (let i=0;i<64;i++) vectors.push(Array.from({{length:4}},()=>{{
            seed^=seed<<13; seed^=seed>>>17; seed^=seed<<5; return seed>>>0;
        }}));
        operations.forEach((operation,index)=>{{ for (let flags=0;flags<16;flags++) for (const values of vectors) {{
            const cpsr=((flags<<28)|0x08000010)>>>0;
            resetMemory({{cpsr,budget:1,sample:1}});
            values.forEach((v,r)=>set(r*4,v));
            const expected=Array.from({{length:15}},(_,r)=>get(r*4));
            let bits=32, result, setFlags=false;
            if (operation.MultiplyLong) {{
                const op=operation.MultiplyLong;
                const extend=v=>op.signed ? BigInt.asIntN(32,BigInt(v)) : BigInt(v);
                result=extend(values[op.left])*extend(values[op.right]);
                if (op.accumulate) result+=BigInt(expected[op.low])+(BigInt(expected[op.high])<<32n);
                result=BigInt.asUintN(64,result); bits=64; setFlags=op.set_flags;
                expected[op.low]=Number(result&0xffffffffn); expected[op.high]=Number(result>>32n);
            }} else if (operation.MultiplyAccumulate) {{
                const op=operation.MultiplyAccumulate;
                result=BigInt.asUintN(32,BigInt(values[op.left])*BigInt(values[op.right])+BigInt(values[op.accumulate]));
                expected[op.destination]=Number(result); setFlags=op.set_flags;
            }} else {{
                const op=operation.Alu;
                result=op.op==='CountLeadingZeros' ? BigInt(Math.clz32(op.right.value.Immediate ?? values[0]))
                    : BigInt.asUintN(32,BigInt(values[0])*BigInt(values[1]));
                expected[op.destination]=Number(result); setFlags=op.set_flags;
            }}
            const nz=(result===0n ? 0x40000000 : 0) | (result>>(BigInt(bits)-1n) ? 0x80000000 : 0);
            const expectedCpsr=setFlags ? ((cpsr&0x3fffffff)|nz)>>>0 : cpsr;
            assert.equal(instance.exports[`region_${{index}}`](F,A),2,`operation ${{index}}`);
            assert.deepEqual(Array.from({{length:15}},(_,r)=>get(r*4)),expected,JSON.stringify(operation));
            assert.equal(get(64),expectedCpsr,JSON.stringify({{operation,values,cpsr}}));
            assert.equal(get(60),0x1004); assert.equal(get(80),1); assert.equal(get(72),0); assert.equal(get(76),0);
            assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0); assert.equal(data.getUint32(164,true),0);
            assert.deepEqual(samples,[[A,0x1000,cpsr,77]]);
        }} }});
    "#,
            serde_json::to_string(&operations).unwrap()
        ),
    );
}

#[test]
fn status_access_preserves_non_nzcv_bits_and_nop_retires_without_memory() {
    for mask in [0, 0x8000_0000, 0x4000_0000, 0x2000_0000, 0x1000_0000, 0xf000_0000] {
        for source in [Value::Register(0), Value::Immediate(0xa5ff_ffff)] {
            let mut input = request(vec![
                Instruction {
                    pc: 0x1000,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::WriteStatus { value: source, mask },
                },
                Instruction {
                    pc: 0x1004,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::ReadStatus { destination: 7 },
                },
                Instruction {
                    pc: 0x1008,
                    size: 4,
                    condition: Condition::Always,
                    operation: Operation::Nop,
                },
            ]);
            input.regions[0].ir.entry.thumb = false;
            run(
                &input,
                &format!(
                    r#"
                for (const cpsr of [0x08000010,0xf80000d0]) for (const v of [0,0xffffffff,0x50000000]) {{
                    resetMemory({{cpsr,budget:3,sample:3}}); set(0,v);
                    const expected=((cpsr&~{mask})|({value}&{mask}))>>>0;
                    assert.equal(entry(F,A),2); assert.equal(get(64),expected); assert.equal(get(28),expected);
                    assert.equal(get(0),v); assert.equal(get(80),3); assert.equal(get(60),0x100c); assert.equal(get(76),0);
                    assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0); assert.equal(data.getUint32(164,true),0);
                    assert.deepEqual(samples,[[A,0x1008,expected,expected]]);
                }}
            "#,
                    value = if matches!(source, Value::Register(_)) { "v" } else { "0xa5ffffff" }
                ),
            );
        }
    }
}

#[test]
fn cpsr_updates_reach_following_conditions_carry_reads_and_pre_instruction_samples() {
    for (operation, expected) in [
        (
            alu(0x1000, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), true).operation,
            0x6800_0030_u32,
        ),
        (
            alu(0x1000, AluOp::Multiply, Some(0), Value::Register(0), Value::Immediate(0), true).operation,
            0x7800_0030,
        ),
        (
            Operation::MultiplyAccumulate {
                destination: 0,
                left: 0,
                right: 1,
                accumulate: 2,
                set_flags: true,
            },
            0x7800_0030,
        ),
        (
            Operation::MultiplyLong {
                low: 0,
                high: 1,
                left: 0,
                right: 1,
                signed: false,
                accumulate: false,
                set_flags: true,
            },
            0x3800_0030,
        ),
        (
            Operation::WriteStatus {
                value: Value::Immediate(0x5000_0000),
                mask: 0xf000_0000,
            },
            0x5800_0030,
        ),
    ] {
        let mut conditional = alu(0x1006, AluOp::Move, Some(4), Value::Immediate(0), Value::Immediate(123), false);
        conditional.condition = Condition::Eq;
        run(
            &request(vec![
                Instruction {
                    pc: 0x1000,
                    size: 2,
                    condition: Condition::Always,
                    operation,
                },
                Instruction {
                    pc: 0x1002,
                    size: 2,
                    condition: Condition::Always,
                    operation: Operation::ReadStatus { destination: 7 },
                },
                alu(0x1004, AluOp::AddCarry, Some(3), Value::Immediate(0), Value::Immediate(0), false),
                conditional,
                Instruction {
                    pc: 0x1008,
                    size: 2,
                    condition: Condition::Always,
                    operation: Operation::Branch {
                        target: Value::Immediate(0x2000),
                        link: None,
                        exchange: true,
                    },
                },
            ]),
            &format!(
                r#"
                for (let stop=2;stop<=5;stop++) {{
                    resetMemory({{cpsr:0xb8000030,sample:stop,entryPc:0x8000}});
                    set(0,0xffffffff); set(4,1); set(8,1);
                    assert.equal(entry(F,A),1);
                    assert.equal(get(64),stop===5 ? ({expected}&~0x20)>>>0 : {expected});
                    assert.equal(get(28),{expected});
                    assert.equal(get(12),stop>=3 ? ({expected}>>>29)&1 : 0);
                    assert.equal(get(16),stop>=4 && ({expected}&0x40000000) ? 123 : 0);
                    assert.equal(get(60),stop===5 ? 0x2000 : 0x1000+2*stop);
                    assert.equal(get(80),stop); assert.equal(get(72),100-stop); assert.equal(get(76),0);
                    assert.equal(get(92),stop===5 ? 0x2000 : 0x8000);
                    assert.equal(data.getUint32(188,true),0x8000);
                    assert.deepEqual(samples,[[A,0x1000+2*(stop-1),{expected},stop===2 ? 77 : {expected}]]);
                }}
                "#,
            ),
        );
    }
}

#[test]
fn multiple_transfers_execute_all_address_modes_and_capture_original_registers() {
    let mut operations = Vec::new();
    for base in [0, 3, 13] {
        for registers in [0x008a_u16, 0xffff, 1 << base, 1 << 14, 1 << 15] {
            for load in [false, true] {
                for increment in [false, true] {
                    for before in [false, true] {
                        for write_back in [false, true] {
                            if write_back && registers & (1 << base) != 0 && (load || registers.trailing_zeros() != u32::from(base)) {
                                continue;
                            }
                            operations.push(Operation::MultipleTransfer {
                                base,
                                registers,
                                increment,
                                before,
                                write_back,
                                load,
                            });
                        }
                    }
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
        operations.forEach(({{MultipleTransfer:op}},index)=>{{ for (const base of [0x2080,0x2f80]) {{
            resetMemory({{cpsr:0xf8000010,budget:1,sample:1}});
            for (let r=0;r<15;r++) set(r*4,0x11000000+r*0x10101);
            set(op.base*4,base);
            const original=Array.from({{length:15}},(_,r)=>get(r*4)), expected=original.slice();
            const registers=Array.from({{length:16}},(_,r)=>r).filter(r=>op.registers&(1<<r));
            const count=registers.length, step=op.increment ? 4 : -4;
            const addresses=Array.from({{length:count}},(_,i)=>base+step*(i+Number(op.before))).sort((a,b)=>a-b);
            registers.forEach((r,i)=>data.setUint32(addresses[i]-0x1000,r===15 ? 0x4001 : 0x82000000+r*0x101,true));
            const expectedMemory=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
            const view=new DataView(expectedMemory.buffer);
            registers.forEach((r,i)=>{{
                const address=addresses[i];
                const v=op.load ? view.getUint32(address-0x2000,true) : r===15 ? 0x100c : original[r];
                if (op.load && r!==15) expected[r]=v;
                if (!op.load) view.setUint32(address-0x2000,v,true);
            }});
            if (op.write_back) expected[op.base]=base+step*count;
            const loadedPc=op.load && registers.includes(15);
            assert.equal(instance.exports[`region_${{index}}`](F,A),2,JSON.stringify(op));
            assert.deepEqual(Array.from({{length:15}},(_,r)=>get(r*4)),expected,JSON.stringify(op));
            assert.equal(get(60),loadedPc ? 0x4000 : 0x1004); assert.equal(get(64),loadedPc ? 0xf8000030 : 0xf8000010);
            assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),expectedMemory);
            assert.equal(data.getUint32(164,true),1); assert.equal(data.getUint32(168,true),addresses[0]);
            assert.equal(data.getUint32(172,true),count); assert.equal(data.getUint32(180,true),A);
            assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0);
            assert.equal(data.getUint32(176,true),0);
            assert.equal(get(80),1); assert.equal(get(72),0); assert.equal(get(76),0);
            assert.deepEqual(samples,[[A,0x1000,0xf8000010,original[7]]]);
        }} }});
    "#,
            serde_json::to_string(&operations).unwrap()
        ),
    );
}

#[test]
fn word_transfers_use_both_borrowed_spans_without_scalar_helpers() {
    for load in [false, true] {
        let input = request(vec![Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: Operation::MultipleTransfer {
                base: 0,
                registers: 0x008e,
                increment: true,
                before: false,
                write_back: true,
                load,
            },
        }]);
        run(
            &input,
            &format!(
                r#"
            for (const split of [4,8,12]) {{
                resetMemory({{sample:1,budget:1}}); set(0,0x2080);
                data.setUint32(184,split,true);
                const registers=[1,2,3,7], expected=[0x12345678,0x89abcdef,0xfedcba98,0x76543210];
                const positions=registers.map((_,i)=>i*4<split ? 0x1080+i*4 : 0x4000+i*4-split);
                registers.forEach((r,i)=>{{ set(r*4,{load} ? 0 : expected[i]); if ({load}) data.setUint32(positions[i],expected[i],true); }});
                const previousR7=get(28);
                assert.equal(entry(F,A),2); assert.equal(get(0),0x2090); assert.equal(get(60),0x1002);
                assert.equal(get(80),1); assert.equal(get(72),0); assert.equal(get(76),0);
                assert.deepEqual(registers.map(r=>get(r*4)),expected);
                assert.deepEqual(positions.map(p=>data.getUint32(p,true)),expected);
                assert.deepEqual(new Uint8Array(memory.buffer,0x1080+split,16-split),new Uint8Array(16-split).fill(0xa5));
                assert.equal(data.getUint32(164,true),1); assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0);
                assert.deepEqual(samples,[[A,0x1000,0x30,previousR7]]);
            }}
        "#
            ),
        );
    }
}

#[test]
fn thumb_stack_transfers_and_listed_base_loads_retire_once() {
    let push = Operation::MultipleTransfer {
        base: 13,
        registers: 1 << 14,
        increment: false,
        before: true,
        write_back: true,
        load: false,
    };
    let pop = Operation::MultipleTransfer {
        base: 13,
        registers: 1 << 15,
        increment: true,
        before: false,
        write_back: true,
        load: true,
    };
    let input = request(vec![
        Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: push,
        },
        Instruction {
            pc: 0x1002,
            size: 2,
            condition: Condition::Always,
            operation: pop,
        },
    ]);
    run(
        &input,
        r#"
        for (const target of [0x4001,0x4000]) {
            resetMemory({budget:2,sample:2,cpsr:0xf8000030}); set(52,0x2080); set(56,target);
            assert.equal(entry(F,A),2); assert.equal(get(52),0x2080); assert.equal(get(56),target);
            assert.equal(get(60),0x4000); assert.equal(get(64),target&1 ? 0xf8000030 : 0xf8000010);
            assert.equal(data.getUint32(0x107c,true),target); assert.equal(data.getUint32(164,true),2);
            assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0);
            assert.equal(get(80),2); assert.equal(get(72),0); assert.equal(get(76),0);
            assert.deepEqual(samples,[[A,0x1002,0xf8000030,77]]);
        }
        resetMemory({budget:1}); set(52,0x2080); set(56,0x4001);
        assert.equal(entry(F,A),2); assert.equal(get(52),0x207c); assert.equal(get(60),0x1002); assert.equal(get(80),1);
        assert.equal(data.getUint32(164,true),1); assert.equal(data.getUint32(152,true),0);
    "#,
    );
    run(
        &request(vec![Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: Operation::MultipleTransfer {
                base: 1,
                registers: 0x008a,
                increment: true,
                before: false,
                write_back: false,
                load: true,
            },
        }]),
        r#"
        resetMemory({sample:1}); set(4,0x2000);
        [0xdeadbeef,0x12345678,0x87654321].forEach((v,i)=>data.setUint32(4096+4*i,v,true));
        assert.equal(entry(F,A),1); assert.equal(get(4),0xdeadbeef); assert.equal(get(12),0x12345678); assert.equal(get(28),0x87654321);
        assert.equal(get(80),1); assert.equal(get(60),0x1002); assert.equal(data.getUint32(164,true),1);
        assert.equal(data.getUint32(152,true),0); assert.deepEqual(samples,[[A,0x1000,0x30,77]]);
    "#,
    );
}

#[test]
fn double_transfers_capture_offsets_and_write_back_after_both_words() {
    let mut operations = Vec::new();
    for load in [false, true] {
        for base in [0, 2] {
            for (pre_index, write_back) in [(true, false), (true, true), (false, true)] {
                if load && base == 2 && write_back {
                    continue;
                }
                for subtract in [false, true] {
                    for offset in [Value::Immediate(8), Value::Register(3)] {
                        operations.push(Operation::DoubleTransfer {
                            register: 2,
                            load,
                            address: Address {
                                base: Value::Register(base),
                                offset: Operand {
                                    value: offset,
                                    shift: Shift::Lsl,
                                    amount: ShiftAmount::Immediate(0),
                                },
                                subtract,
                                pre_index,
                                write_back: write_back.then_some(base),
                            },
                        });
                    }
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
        operations.forEach(({{DoubleTransfer:op}},index)=>{{
            resetMemory({{cpsr:0xf8000010,sample:1}}); set(8,0x12345678); set(12,8); set(op.address.base.Register*4,0x2080);
            const expected=Array.from({{length:15}},(_,r)=>get(r*4));
            const updated=0x2080+(op.address.subtract ? -8 : 8), address=op.address.pre_index ? updated : 0x2080;
            data.setUint32(address-0x1000,0x87654321,true); data.setUint32(address-0xffc,0xfedcba98,true);
            const expectedMemory=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096)), view=new DataView(expectedMemory.buffer);
            for (let i=0;i<2;i++) {{
                const v=op.load ? view.getUint32(address-0x2000+4*i,true) : expected[2+i];
                if (op.load) expected[2+i]=v; else view.setUint32(address-0x2000+4*i,v,true);
            }}
            if (op.address.write_back!==null) expected[op.address.write_back]=updated;
            assert.equal(instance.exports[`region_${{index}}`](F,A),1);
            assert.deepEqual(Array.from({{length:15}},(_,r)=>get(r*4)),expected,JSON.stringify(op));
            assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),expectedMemory);
            assert.equal(data.getUint32(176,true),0);
            assert.equal(data.getUint32(164,true),1); assert.equal(data.getUint32(168,true),address); assert.equal(data.getUint32(172,true),2);
            assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0);
            assert.equal(get(64),0xf8000010); assert.equal(get(60),0x1004); assert.equal(get(80),1); assert.equal(get(72),99); assert.equal(get(76),0);
            assert.deepEqual(samples,[[A,0x1000,0xf8000010,77]]);
        }});
    "#,
            serde_json::to_string(&operations).unwrap()
        ),
    );
}

#[test]
fn range_rejection_precedes_every_transfer_effect_and_preserves_retired_prefixes() {
    let mut operations = Vec::new();
    for load in [false, true] {
        operations.push(Operation::MultipleTransfer {
            base: 0,
            registers: 0x800e,
            increment: true,
            before: false,
            write_back: true,
            load,
        });
        operations.push(Operation::DoubleTransfer {
            register: 2,
            load,
            address: Address {
                base: Value::Register(0),
                offset: Operand {
                    value: Value::Immediate(8),
                    shift: Shift::Lsl,
                    amount: ShiftAmount::Immediate(0),
                },
                subtract: false,
                pre_index: false,
                write_back: Some(0),
            },
        });
    }
    operations.push(Operation::Swap {
        destination: 0,
        address: 0,
        value: 1,
        width: Width::Word,
    });
    for operation in operations {
        let words = match operation {
            Operation::MultipleTransfer { .. } => 4,
            Operation::DoubleTransfer { .. } => 2,
            _ => 1,
        };
        let mut prefix = alu(0x1000, AluOp::Add, Some(6), Value::Register(6), Value::Immediate(1), false);
        prefix.size = 4;
        let mut input = request(vec![
            prefix,
            Instruction {
                pc: 0x1004,
                size: 4,
                condition: Condition::Always,
                operation,
            },
        ]);
        input.regions[0].ir.entry.thumb = false;
        run(
            &input,
            &format!(
                r#"
            const addresses=[0x1ffc,0x3000,0x2001,0x2002,0x2003,0xfffffffc];
            for (let valid=1;valid<{words};valid++) addresses.push(0x3000-4*valid);
            for (const pc of [0x1000,0x1004]) for (const [address,status] of [...addresses.map(a=>[a,0]),[0x2000,1],[0x2000,2]]) {{
                resetMemory({{pc,cpsr:0xf8000010,sample:pc===0x1000 ? 2 : 1}});
                set(0,address); set(4,0x12345678); set(88,0xdeadbeef); data.setUint32(148,status,true);
                const before=Array.from(new Uint32Array(memory.buffer,F,24));
                const expected=before.slice(), retired=pc===0x1000 ? 1 : 0;
                expected[6]+=retired; expected[15]=0x1004; expected[18]-=retired; expected[19]-=retired; expected[20]+=retired;
                const bytes=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
                assert.equal(entry(F,A),4);
                assert.deepEqual(Array.from(new Uint32Array(memory.buffer,F,24)),expected);
                assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),bytes);
                assert.equal(data.getUint32(164,true),1); assert.equal(data.getUint32(168,true),address); assert.equal(data.getUint32(172,true),{words});
                assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0); assert.equal(data.getUint32(176,true),0);
                // Preparation may stage a sample, but the rejected instruction has not retired.
                assert.deepEqual(samples,[[A,0x1004,0xf8000010,77]]);
            }}
        "#
            ),
        );
    }
}

#[test]
fn swaps_read_before_writing_with_aliased_registers_and_decline_without_effects() {
    for width in [Width::Byte, Width::Word] {
        for (destination, source) in [(2, 1), (0, 1), (1, 1), (0, 0), (2, 0)] {
            let mut input = request(vec![Instruction {
                pc: 0x1000,
                size: 4,
                condition: Condition::Always,
                operation: Operation::Swap {
                    destination,
                    address: 0,
                    value: source,
                    width,
                },
            }]);
            input.regions[0].ir.entry.thumb = false;
            let bytes = if width == Width::Byte { 1 } else { 4 };
            run(
                &input,
                &format!(
                    r#"
                const address={address}, width={bytes};
                resetMemory({{cpsr:0xf8000010,budget:1,sample:1}}); set(0,address); set(4,0x12345678);
                data.setUint32(address-0x1000,0x89abcdef,true);
                const expected=Array.from({{length:15}},(_,r)=>get(r*4)), value=expected[{source}];
                const loaded=width===1 ? 0xef : 0x89abcdef; expected[{destination}]=loaded;
                const expectedMemory=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096)), view=new DataView(expectedMemory.buffer);
                if (width===1) view.setUint8(address-0x2000,value); else view.setUint32(address-0x2000,value,true);
                assert.equal(entry(F,A),2); assert.deepEqual(Array.from({{length:15}},(_,r)=>get(r*4)),expected);
                assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),expectedMemory);
                if (width===1) assert.deepEqual(Array.from({{length:2}},(_,i)=>Array.from(new Uint32Array(memory.buffer,8192+16*i,4))),[[0,address,width,loaded],[1,address,width,value]]);
                assert.equal(data.getUint32(164,true),Number(width===4));
                assert.equal(data.getUint32(152,true),Number(width===1)); assert.equal(data.getUint32(156,true),Number(width===1));
                assert.equal(get(64),0xf8000010); assert.equal(get(60),0x1004); assert.equal(get(80),1); assert.equal(get(72),0); assert.equal(get(76),0);
                assert.deepEqual(samples,[[A,0x1000,0xf8000010,77]]);
                for (const [address,status] of [[0x3000,0],[0x1fff,0],[0x2000,1]]) {{
                    resetMemory({{cpsr:0xf8000010}}); set(0,address); set(4,0x12345678); set(88,0xdeadbeef); data.setUint32(148,status,true);
                    const before=Array.from(new Uint32Array(memory.buffer,F,24)), memoryBefore=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
                    assert.equal(entry(F,A),4); assert.deepEqual(Array.from(new Uint32Array(memory.buffer,F,24)),before);
                    assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),memoryBefore); assert.equal(data.getUint32(156,true),0);
                    assert.equal(data.getUint32(152,true),Number(width===1)); assert.equal(data.getUint32(164,true),Number(width===4));
                    assert.equal(data.getUint32(176,true),0); assert.equal(samples.length,0);
                }}
            "#,
                    address = if width == Width::Byte { 0x2001 } else { 0x2000 }
                ),
            );
        }
    }
}

#[test]
fn new_operations_with_false_conditions_retire_without_accesses_or_register_writes() {
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
    let operations = vec![
        alu(0x1000, AluOp::CountLeadingZeros, Some(7), Value::Immediate(0), Value::Register(0), false).operation,
        alu(0x1000, AluOp::Move, Some(15), Value::Immediate(0), Value::Register(0), false).operation,
        Operation::MultiplyAccumulate {
            destination: 7,
            left: 0,
            right: 1,
            accumulate: 7,
            set_flags: true,
        },
        Operation::MultiplyLong {
            low: 7,
            high: 8,
            left: 0,
            right: 1,
            signed: true,
            accumulate: true,
            set_flags: true,
        },
        Operation::ReadStatus { destination: 7 },
        Operation::WriteStatus {
            value: Value::Register(0),
            mask: 0xf000_0000,
        },
        Operation::MultipleTransfer {
            base: 0,
            registers: 0x8082,
            increment: true,
            before: false,
            write_back: true,
            load: true,
        },
        Operation::MultipleTransfer {
            base: 0,
            registers: 0x8082,
            increment: false,
            before: true,
            write_back: true,
            load: false,
        },
        Operation::DoubleTransfer {
            register: 2,
            address,
            load: true,
        },
        Operation::DoubleTransfer {
            register: 2,
            address,
            load: false,
        },
        Operation::Swap {
            destination: 7,
            address: 0,
            value: 1,
            width: Width::Word,
        },
        Operation::Swap {
            destination: 7,
            address: 0,
            value: 1,
            width: Width::Byte,
        },
        Operation::Load {
            destination: 15,
            address,
            width: Width::Word,
            signed: false,
        },
        Operation::Nop,
    ];
    let mut input = request(Vec::new());
    input.regions.clear();
    for operation in operations {
        let mut region = request(vec![Instruction {
            pc: 0x1000,
            size: 4,
            condition: Condition::Eq,
            operation,
        }]);
        region.regions[0].ir.entry.thumb = false;
        input.regions.extend(region.regions);
    }
    run(
        &input,
        r#"
        for (const entry of Object.values(instance.exports)) {
            resetMemory({cpsr:0xb8000010,sample:1}); set(0,0x2001); set(4,0xdeadbeef); data.setUint32(148,2,true);
            const expected=Array.from(new Uint32Array(memory.buffer,F,24)); expected[15]+=4; expected[18]--; expected[19]--; expected[20]++;
            const expectedMemory=Uint8Array.from(new Uint8Array(memory.buffer,4096,4096));
            assert.equal(entry(F,A),1); assert.deepEqual(Array.from(new Uint32Array(memory.buffer,F,24)),expected);
            assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),expectedMemory);
            assert.equal(data.getUint32(164,true),0); assert.equal(data.getUint32(152,true),0); assert.equal(data.getUint32(156,true),0);
            assert.deepEqual(samples,[[A,0x1000,0xb8000010,77]]);
        }
    "#,
    );
}

#[test]
fn load_pc_interworks_and_applies_boundaries_after_whole_instruction_retirement() {
    for multiple in [false, true] {
        let operation = if multiple {
            Operation::MultipleTransfer {
                base: 0,
                registers: 0x8082,
                increment: true,
                before: false,
                write_back: true,
                load: true,
            }
        } else {
            Operation::Load {
                destination: 15,
                width: Width::Word,
                signed: false,
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
            }
        };
        let mut input = request(vec![Instruction {
            pc: 0x1000,
            size: 4,
            condition: Condition::Always,
            operation,
        }]);
        input.regions[0].ir.entry.thumb = false;
        run(
            &input,
            &format!(
                r#"
            const words={words};
            for (const target of [0x4000,0x4001,8,9]) for (const [options,boundary] of [
                [{{}},0], [{{sample:1}},1], [{{budget:1,sample:1}},2], [{{end:target&~1,budget:1,sample:1}},3]
            ]) {{
                resetMemory({{cpsr:0xf8000010,...options}}); set(0,0x2000);
                [0xabcdef01,0x23456789].forEach((v,i)=>data.setUint32(4096+4*i,v,true));
                data.setUint32(4096+4*(words-1),target,true);
                assert.equal(entry(F,A),target<0x1000 ? 6 : boundary);
                assert.equal(get(60),target&~1); assert.equal(get(64),target&1 ? 0xf8000030 : 0xf8000010);
                assert.equal(get(0),0x2000+4*words); assert.equal(get(80),1);
                assert.equal(get(72),(options.budget??100)-1); assert.equal(get(76),(options.sample??100)-1);
                assert.equal(data.getUint32(152,true),Number(words===1)); assert.equal(data.getUint32(156,true),0);
                assert.equal(data.getUint32(164,true),Number(words>1));
                if (words>1) {{ assert.equal(get(4),0xabcdef01); assert.equal(get(28),0x23456789); }}
                if (options.sample===1) assert.deepEqual(samples,[[A,0x1000,0xf8000010,77]]);
                else assert.equal(samples.length,0);
                assert.equal(get(84),target<0x1000 ? target&~1 : 0);
            }}
        "#,
                words = if multiple { 3 } else { 1 }
            ),
        );
    }
}

#[test]
fn thumb_call_pairs_retire_as_one_instruction_and_do_not_expose_the_suffix() {
    for exchange in [false, true] {
        let mut input = request(vec![
            alu(0x1000, AluOp::Move, Some(0), Value::Immediate(0), Value::Immediate(1), false),
            Instruction {
                pc: 0x1002,
                size: 4,
                condition: Condition::Always,
                operation: Operation::Branch {
                    target: Value::Immediate(0x2000),
                    link: Some(0x1007),
                    exchange,
                },
            },
        ]);
        input.regions[0].ir.blocks.push(BasicBlock {
            instructions: vec![alu(0x1006, AluOp::Move, Some(1), Value::Immediate(0), Value::Immediate(2), false)],
        });
        run(
            &input,
            &format!(
                r#"
            reset({{budget:2,sample:2}}); assert.equal(entry(F,A),2); assert.equal(get(0),1); assert.equal(get(4),0);
            assert.equal(get(60),0x2000); assert.equal(get(56),0x1007); assert.equal(get(64),{cpsr}); assert.equal(get(80),2);
            assert.equal(get(76),0); assert.deepEqual(samples,[[A,0x1002,0x30,77]]);
            reset({{pc:0x1002,budget:1,sample:1}}); assert.equal(entry(F,A),2); assert.equal(get(80),1); assert.equal(get(56),0x1007);
            reset({{pc:0x1004,sample:1}}); const before=Array.from(new Uint32Array(memory.buffer,F,24));
            assert.equal(entry(F,A),0); assert.deepEqual(Array.from(new Uint32Array(memory.buffer,F,24)),before); assert.equal(samples.length,0);
            reset({{pc:0x1006,budget:1}}); assert.equal(entry(F,A),2); assert.equal(get(4),2); assert.equal(get(80),1);
        "#,
                cpsr = if exchange { 0x10 } else { 0x30 }
            ),
        );
    }
    for condition in [Condition::Always, Condition::Eq] {
        run(
            &request(vec![Instruction {
                pc: 0x1000,
                size: 4,
                condition,
                operation: Operation::Branch {
                    target: Value::Immediate(0x1004),
                    link: Some(0x1005),
                    exchange: false,
                },
            }]),
            &format!(
                r#"
                resetMemory({{budget:1,sample:1,entryPc:0x8000}});
                assert.equal(entry(F,A),2); assert.equal(get(60),0x1004); assert.equal(get(64),0x30);
                assert.equal(get(56),{link}); assert.equal(get(80),1); assert.equal(get(92),0x1004);
                assert.equal(data.getUint32(188,true),0x8000); assert.deepEqual(samples,[[A,0x1000,0x30,77]]);
                "#,
                link = if condition == Condition::Always { 0x1005 } else { 0 },
            ),
        );
    }
}

#[test]
fn arm_store_pc_uses_the_instruction_address_plus_twelve() {
    for source in [Value::Immediate(0x100c), Value::Register(15)] {
        let mut input = request(vec![Instruction {
            pc: 0x1000,
            size: 4,
            condition: Condition::Always,
            operation: Operation::Store {
                value: source,
                width: Width::Word,
                address: Address {
                    base: Value::Register(0),
                    offset: Operand {
                        value: Value::Immediate(0),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    subtract: false,
                    pre_index: true,
                    write_back: None,
                },
            },
        }]);
        input.regions[0].ir.entry.thumb = false;
        run(
            &input,
            r#"
            resetMemory({cpsr:0x10,budget:1}); set(0,0x2000);
            assert.equal(entry(F,A),2); assert.equal(data.getUint32(4096,true),0x100c);
            assert.equal(get(60),0x1004); assert.equal(get(80),1); assert.equal(data.getUint32(156,true),1);
            assert.equal(data.getUint32(164,true),0);
        "#,
        );
    }
}

#[test]
fn narrow_pc_stores_reject_register_sources_and_keep_literal_values() {
    for thumb in [false, true] {
        for width in [Width::Byte, Width::Half] {
            let mut input = request(vec![Instruction {
                pc: 0x1000,
                size: if thumb { 2 } else { 4 },
                condition: Condition::Always,
                operation: Operation::Store {
                    value: Value::Register(15),
                    width,
                    address: Address {
                        base: Value::Register(0),
                        offset: Operand {
                            value: Value::Immediate(0),
                            shift: Shift::Lsl,
                            amount: ShiftAmount::Immediate(0),
                        },
                        subtract: false,
                        pre_index: true,
                        write_back: None,
                    },
                },
            }]);
            input.regions[0].ir.entry.thumb = thumb;
            assert_eq!(codegen::compile(&input).err().as_deref(), Some("invalid instruction operands"));
            if let Operation::Store { value, .. } = &mut input.regions[0].ir.blocks[0].instructions[0].operation {
                *value = Value::Immediate(0x100c);
            }
            run(
                &input,
                &format!(
                    r#"
                    resetMemory({{cpsr:{cpsr},budget:1}}); set(0,0x2000);
                    assert.equal(entry(F,A),2); assert.equal(data.getUint32(4096,true),{stored});
                    assert.equal(get(0),0x2000); assert.equal(get(60),{next}); assert.equal(get(64),{cpsr});
                    assert.equal(get(80),1); assert.equal(get(72),0); assert.equal(get(76),99);
                    assert.equal(data.getUint32(156,true),1); assert.equal(data.getUint32(152,true),0);
                    assert.equal(data.getUint32(164,true),0); assert.equal(samples.length,0);
                    "#,
                    cpsr = if thumb { 0x30 } else { 0x10 },
                    next = if thumb { 0x1002 } else { 0x1004 },
                    stored = if width == Width::Byte { 0xa5a5_a50c_u32 } else { 0xa5a5_100c },
                ),
            );
        }
    }
}

#[test]
fn boundary_fields_are_loaded_once_per_invocation_and_reloaded_on_resume() {
    let input = request(
        (0..3)
            .map(|index| alu(0x1000 + index * 2, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false))
            .collect(),
    );
    run(
        &input,
        r#"
        reset({end:0x1002});
        assert.equal(entry(F,A),3); assert.equal(get(0),1); assert.equal(get(80),1);
        set(68,0x1004);
        assert.equal(entry(F,A),3); assert.equal(get(0),2); assert.equal(get(80),2);
        set(68,0x1006); set(72,1); set(76,1);
        assert.equal(entry(F,A),3); assert.equal(get(0),3); assert.equal(get(80),3);
        assert.equal(get(72),0); assert.equal(get(76),0);
        assert.deepEqual(samples,[[A,0x1004,0x30,77]]);
        for (const [options,exit] of [[{end:0x1000,budget:0,sample:0},3],[{budget:0,sample:0},2],[{sample:0},1]]) {
            reset(options); assert.equal(entry(F,A),exit); assert.equal(get(80),0); assert.equal(samples.length,0);
        }
        reset({pc:8,end:8,budget:0,sample:0});
        assert.equal(entry(F,A),6); assert.equal(get(84),8); assert.equal(get(80),0);
        assert.equal(samples.length,0);
        "#,
    );
    let artifact = codegen::compile(&input).unwrap();
    for offset in [68, 64] {
        let mut load = Function::new([]);
        load.instructions().local_get(0).i32_load(MemArg {
            offset,
            align: 2,
            memory_index: 0,
        });
        // This fixture has no memory operations or constants containing the load encoding.
        let encoding = load.into_raw_body();
        let instruction = &encoding[1..];
        assert_eq!(
            artifact.bytes.windows(instruction.len()).filter(|bytes| *bytes == instruction).count(),
            1,
            "field {offset}"
        );
    }
}

#[test]
fn straight_line_blocks_stop_and_resume_at_every_instruction() {
    for thumb in [false, true] {
        let stride = if thumb { 2 } else { 4 };
        let mut input = request(
            (0..12)
                .map(|index| {
                    let mut instruction = alu(
                        0x1000 + index * stride,
                        AluOp::Add,
                        Some(0),
                        Value::Register(0),
                        Value::Immediate(index + 1),
                        true,
                    );
                    instruction.size = stride as u8;
                    instruction
                })
                .collect(),
        );
        input.regions[0].ir.entry.thumb = thumb;
        let second = input.regions[0].ir.blocks[0].instructions.split_off(6);
        input.regions[0].ir.blocks.push(BasicBlock { instructions: second });
        run(
            &input,
            &format!(
                r#"
                const stride={stride}, cpsr={cpsr}, finalPc=0x1000+12*stride;
                const total=(start,end)=>(end*(end+1)-start*(start+1))/2;
                for (let start=0; start<12; start++) for (let stop=start; stop<=12; stop++) {{
                    const retired=stop-start, pc=0x1000+stop*stride;
                    for (const [options,exit] of [[{{budget:retired}},2],[{{sample:retired}},1],[{{end:pc}},3]]) {{
                        reset({{pc:0x1000+start*stride,cpsr,...options}});
                        assert.equal(entry(F,A),exit);
                        assert.equal(get(0),total(start,stop));
                        assert.equal(get(60),pc); assert.equal(get(64),cpsr);
                        assert.equal(get(80),retired);
                        assert.equal(get(72),(options.budget??100)-retired);
                        assert.equal(get(76),(options.sample??100)-retired);
                        assert.deepEqual(samples,options.sample>0 ? [[A,pc-stride,cpsr,77]] : []);
                        set(68,finalPc); set(72,100); set(76,100);
                        assert.equal(entry(F,A),3);
                        assert.equal(get(0),total(start,12));
                        assert.equal(get(60),finalPc); assert.equal(get(80),12-start);
                    }}
                }}
                for (const previous of [7,0xfffffffe,0xffffffff]) {{
                    reset({{cpsr,budget:3}}); set(80,previous);
                    assert.equal(entry(F,A),2); assert.equal(get(80),(previous+3)>>>0);
                    assert.equal(get(72),0); assert.equal(get(76),97); assert.equal(get(0),6);
                }}
                "#,
                cpsr = if thumb { 0x30 } else { 0x10 },
            ),
        );
    }
}

#[test]
fn backend_traps_preserve_completed_guest_stores_without_a_logical_exit() {
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
    let input = request(vec![
        Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Always,
            operation: Operation::Store {
                value: Value::Register(2),
                address,
                width: Width::Word,
            },
        },
        alu(0x1002, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false),
        Instruction {
            pc: 0x1004,
            size: 2,
            condition: Condition::Always,
            operation: Operation::Load {
                destination: 3,
                address,
                width: Width::Word,
                signed: false,
            },
        },
    ]);
    run(
        &input,
        r#"
        resetMemory(); set(4,0x2080); set(8,0xdeadbeef); data.setUint32(148,3,true);
        assert.throws(()=>entry(F,A),WebAssembly.RuntimeError);
        assert.equal(data.getUint32(0x1080,true),0xdeadbeef);
        assert.equal(get(0),1); assert.equal(get(12),0); assert.equal(get(60),0x1004);
        assert.equal(get(72),100); assert.equal(get(76),100); assert.equal(get(80),0);
        assert.equal(data.getUint32(152,true),1); assert.equal(data.getUint32(156,true),1);
        assert.equal(data.getUint32(164,true),0); assert.equal(samples.length,0);
        "#,
    );
}

#[test]
fn memory_exits_keep_completed_prefixes_from_later_blocks() {
    for block_index in [1, 2] {
        for instruction_index in 0..3 {
            for multiple in [false, true] {
                let mut input = request(Vec::new());
                input.regions[0].ir.blocks.clear();
                let mut pc = 0x1000;
                for size in [2, 3, 4] {
                    let instructions = (0..size)
                        .map(|_| {
                            let instruction = alu(pc, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false);
                            pc += 2;
                            instruction
                        })
                        .collect();
                    input.regions[0].ir.blocks.push(BasicBlock { instructions });
                }
                let failed = &mut input.regions[0].ir.blocks[block_index].instructions[instruction_index];
                failed.operation = if multiple {
                    Operation::MultipleTransfer {
                        base: 1,
                        registers: 4,
                        increment: true,
                        before: false,
                        write_back: false,
                        load: true,
                    }
                } else {
                    Operation::Load {
                        destination: 2,
                        address: Address {
                            base: Value::Register(1),
                            offset: Operand {
                                value: Value::Immediate(0),
                                shift: Shift::Lsl,
                                amount: ShiftAmount::Immediate(0),
                            },
                            subtract: false,
                            pre_index: true,
                            write_back: None,
                        },
                        width: Width::Word,
                        signed: false,
                    }
                };
                let failed_pc = failed.pc;
                let prefix = (failed_pc - 0x1000) / 2;
                run(
                    &input,
                    &format!(
                        r#"
                        for (const status of [1,2]) {{
                            resetMemory({{sample:{prefix}+1,entryPc:0x8000}}); set(4,0x2080); set(80,7);
                            data.setUint32(148,status,true);
                            const before=new Uint8Array(memory.buffer,4096,4096).slice();
                            assert.equal(entry(F,A),{multiple}||status===1 ? 4 : {fault});
                            assert.equal(get(60),{failed_pc}); assert.equal(get(80),7+{prefix});
                            assert.equal(get(0),{prefix}); assert.equal(get(8),0);
                            assert.equal(get(72),100-{prefix}); assert.equal(get(76),1);
                            assert.equal(samples.length,1);
                            assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),before);
                        }}
                        "#,
                        fault = CompiledExit::GuestFault as u32,
                    ),
                );
            }
        }
    }
}

#[test]
fn taken_calls_dispatch_before_an_internal_callee_and_keep_boundary_priority() {
    for thumb in [false, true] {
        let width = if thumb { 2 } else { 4 };
        let mut input = request(vec![Instruction {
            pc: 0x1000,
            size: 4,
            condition: Condition::Eq,
            operation: Operation::Branch {
                target: Value::Immediate(0x1010),
                link: Some(0x1004 | u32::from(thumb)),
                exchange: false,
            },
        }]);
        input.regions[0].ir.entry.thumb = thumb;
        for (pc, delta, target) in [(0x1004, 10, Value::Immediate(0x2000)), (0x1010, 1, Value::Register(14))] {
            input.regions[0].ir.blocks.push(BasicBlock {
                instructions: vec![
                    Instruction {
                        size: width,
                        ..alu(pc, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(delta), false)
                    },
                    Instruction {
                        pc: pc + u32::from(width),
                        size: width,
                        condition: Condition::Always,
                        operation: Operation::Branch {
                            target,
                            link: None,
                            exchange: matches!(target, Value::Register(_)),
                        },
                    },
                ],
            });
        }
        run(
            &input,
            &format!(
                r#"
                const cpsr={cpsr}, link={link};
                for (const [options,exit] of [[{{}},0],[{{sample:1}},1],[{{budget:1,sample:1}},2],[{{end:0x1010,budget:1,sample:1}},3]]) {{
                    reset({{cpsr:cpsr|0x40000000,entryPc:0x8000,...options}});
                    assert.equal(entry(F,A),exit);
                    assert.equal(get(60),0x1010); assert.equal(get(0),0); assert.equal(get(56),link);
                    assert.equal(get(80),1); assert.equal(get(92),0x1010);
                    assert.equal(samples.length,Number(options.sample===1));
                    set(68,0x2000); set(72,100); set(76,100);
                    assert.equal(entry(F,A),3); assert.equal(get(80),5); assert.equal(get(0),11);
                    assert.equal(get(60),0x2000);
                }}
                reset({{cpsr,end:0x2000}});
                assert.equal(entry(F,A),3); assert.equal(get(0),10); assert.equal(get(80),3);
                assert.equal(get(56),0); assert.equal(get(60),0x2000);
                "#,
                cpsr = if thumb { 0x30 } else { 0x10 },
                link = 0x1004 | u32::from(thumb),
            ),
        );
    }
}

#[test]
fn register_calls_dispatch_at_fallthrough_and_check_target_faults_first() {
    for thumb in [false, true] {
        let size = if thumb { 2 } else { 4 };
        let next = 0x1000 + u32::from(size);
        let mut input = request(vec![Instruction {
            pc: 0x1000,
            size,
            condition: Condition::Always,
            operation: Operation::Branch {
                target: Value::Register(0),
                link: Some(next | u32::from(thumb)),
                exchange: true,
            },
        }]);
        input.regions[0].ir.entry.thumb = thumb;
        input.regions[0].ir.blocks.push(BasicBlock {
            instructions: vec![Instruction {
                size,
                ..alu(next, AluOp::Add, Some(1), Value::Register(1), Value::Immediate(1), false)
            }],
        });
        run(
            &input,
            &format!(
                r#"
                const cpsr={cpsr}, thumb={thumb}, next={next};
                for (const target of [next|Number(thumb),0x1000|Number(thumb),0x1000|Number(!thumb)]) {{
                    reset({{cpsr,entryPc:0x8000}}); set(0,target);
                    assert.equal(entry(F,A),0); assert.equal(get(80),1); assert.equal(get(4),0);
                    assert.equal(get(56),next|Number(thumb)); assert.equal(get(60),target&~1);
                    assert.equal(get(64),(cpsr&~0x20)|((target&1)<<5));
                }}
                reset({{cpsr,end:8,budget:1,sample:1,entryPc:0x8000}}); set(0,8);
                assert.equal(entry(F,A),{fault}); assert.equal(get(80),1); assert.equal(get(60),8);
                assert.equal(get(84),8); assert.equal(get(72),0); assert.equal(get(76),0);
                assert.equal(samples.length,1);
                "#,
                cpsr = if thumb { 0x30 } else { 0x10 },
                fault = CompiledExit::GuestFault as u32,
            ),
        );
    }
}

#[test]
fn sampled_transfers_keep_the_pre_instruction_entry_and_publish_the_next_entry() {
    for thumb in [false, true] {
        let size = if thumb { 2 } else { 4 };
        for exchange in [false, true] {
            let mut input = request(vec![Instruction {
                pc: 0x1000,
                size,
                condition: Condition::Eq,
                operation: Operation::Branch {
                    target: Value::Register(0),
                    link: None,
                    exchange,
                },
            }]);
            input.regions[0].ir.entry.thumb = thumb;
            run(
                &input,
                &format!(
                    r#"
                    const initialCpsr = {cpsr}, sequential = 0x1000 + {size};
                    for (const taken of [false,true]) for (const next of [sequential,0x2000]) {{
                        for (const targetThumb of [false,true]) {{
                            const target = next | Number(targetThumb);
                            const cpsr = initialCpsr | (taken ? 0x40000000 : 0);
                            const changesMode = {exchange} && targetThumb !== {thumb};
                            const aligned = target & (({exchange} ? targetThumb : {thumb}) ? ~1 : ~3);
                            const expectedEntry = taken && (aligned !== sequential || changesMode) ? aligned : 0x8000;
                            for (const [options,exit] of [[{{sample:1}},1],[{{budget:1,sample:1}},2]]) {{
                                reset({{...options,cpsr,entryPc:0x8000}}); set(0,target);
                                assert.equal(entry(F,A),exit); assert.equal(get(80),1);
                                assert.equal(get(92),expectedEntry);
                                assert.equal(data.getUint32(188,true),0x8000);
                                assert.deepEqual(samples,[[A,0x1000,cpsr,77]]);
                            }}
                        }}
                    }}
                    for (const [options,exit] of [[{{budget:0}},2],[{{sample:0}},1],[{{end:0x1000}},3]]) {{
                        reset({{...options,cpsr:initialCpsr,entryPc:0x8000}});
                        assert.equal(entry(F,A),exit); assert.equal(get(80),0); assert.equal(get(92),0x8000);
                        assert.equal(samples.length,0);
                    }}
                    "#,
                    cpsr = if thumb { 0x30 } else { 0x10 },
                ),
            );
        }
    }
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
fn exchange_preserves_instruction_boundaries_and_exception_return_is_rejected() {
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
        assert!(codegen::compile(&request(vec![instruction])).is_err());
    }
}

#[test]
fn omitted_frontier_dispatches_without_retiring_and_keeps_the_branch_alternative() {
    let mut input = request(vec![Instruction {
        pc: 0x1000,
        size: 2,
        condition: Condition::Eq,
        operation: Operation::Branch {
            target: Value::Immediate(0x1004),
            link: None,
            exchange: false,
        },
    }]);
    input.regions[0].ir.blocks.push(BasicBlock {
        instructions: vec![alu(0x1004, AluOp::Add, Some(0), Value::Register(0), Value::Immediate(1), false)],
    });
    run(
        &input,
        r#"
        reset(); assert.equal(entry(F,A),0); assert.equal(get(60),0x1002); assert.equal(get(80),1);
        assert.equal(get(72),99); assert.equal(get(76),99); assert.equal(get(0),0);
        reset({pc:0x1002,sample:1}); const before=Array.from(new Uint8Array(memory.buffer,F,96));
        assert.equal(entry(F,A),0); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,96)),before);
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
fn malformed_operand_forms_are_compile_errors() {
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
            destination: 16,
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
    operations.push(alu(0x1000, AluOp::Move, Some(16), Value::Immediate(0), Value::Register(0), false).operation);
    for operation in operations {
        assert!(
            codegen::compile(&request(vec![Instruction {
                pc: 0x1000,
                size: 2,
                condition: Condition::Always,
                operation,
            }]))
            .is_err()
        );
    }
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
            assert.equal(data.getUint32(164,true),0);
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
                assert.equal(data.getUint32(164,true),0);
                assert.equal(data.getUint32(152,true)+data.getUint32(156,true),1);
                assert.deepEqual(new Uint8Array(memory.buffer,4096,4096),before);
                // This is a staged sample only: the failed instruction has not retired.
                assert.deepEqual(samples,[[A,0x1002,0x30,77]]);
            }
        "#,
        );
    }
}

#[test]
fn completed_store_continues_with_normal_boundary_priority() {
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
        for (const [options,exit] of [[{},0],[{sample:1},1],[{budget:1,sample:1},2],[{end:0x1002,budget:1,sample:1},3]]) {
            resetMemory(options); set(0,0x2000); set(4,0x12345678); set(28,0x2000);
            assert.equal(entry(F,A),exit); assert.equal(data.getUint32(4096,true),0x12345678);
            const retired=exit===0 ? 2 : 1;
            assert.equal(get(0),0x2004); assert.equal(get(12),retired-1); assert.equal(get(60),0x1000+2*retired); assert.equal(get(80),retired);
            assert.equal(get(72),(options.budget??100)-retired); assert.equal(get(76),(options.sample??100)-retired);
            assert.equal(data.getUint32(156,true),1); assert.equal(data.getUint32(164,true),0);
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
        resetMemory({pc:0xfffffffe,end:0,budget:1,sample:1}); set(0,0x2000); set(4,42);
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
        resetMemory(); set(0,0x2001);
        assert.equal(entry(F,A),0); assert.equal(get(0),0x2001); assert.equal(get(12),1); assert.equal(get(80),2);
        assert.equal(data.getUint32(156,true),0); assert.equal(data.getUint32(4096,true),0xa5a5a5a5);
        assert.equal(data.getUint32(164,true),0);
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
        assert.equal(data.getUint32(164,true),0); assert.equal(data.getUint32(152,true),1);
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
        assert.equal(data.getUint32(164,true),0);
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
fn malformed_basic_blocks_are_rejected_without_normalization() {
    let first = Instruction {
        pc: 0x1000,
        size: 2,
        condition: Condition::Always,
        operation: Operation::Nop,
    };
    let next = Instruction { pc: 0x1002, ..first.clone() };
    let mut malformed = vec![
        vec![
            BasicBlock {
                instructions: vec![first.clone()],
            },
            BasicBlock { instructions: vec![] },
        ],
        vec![BasicBlock {
            instructions: vec![first.clone(), Instruction { pc: 0x1004, ..next.clone() }],
        }],
        vec![BasicBlock {
            instructions: vec![next.clone(), first.clone()],
        }],
    ];
    for condition in [Condition::Always, Condition::Eq] {
        for operation in [
            Operation::Branch {
                target: Value::Register(0),
                link: None,
                exchange: false,
            },
            alu(0x1000, AluOp::Move, Some(15), Value::Immediate(0), Value::Register(0), false).operation,
            Operation::Load {
                destination: 15,
                address: Address {
                    base: Value::Register(0),
                    offset: Operand {
                        value: Value::Immediate(0),
                        shift: Shift::Lsl,
                        amount: ShiftAmount::Immediate(0),
                    },
                    subtract: false,
                    pre_index: true,
                    write_back: None,
                },
                width: Width::Word,
                signed: false,
            },
            Operation::MultipleTransfer {
                base: 0,
                registers: 0x8002,
                increment: true,
                before: false,
                write_back: false,
                load: true,
            },
        ] {
            malformed.push(vec![BasicBlock {
                instructions: vec![
                    Instruction {
                        condition,
                        operation,
                        ..first.clone()
                    },
                    next.clone(),
                ],
            }]);
        }
    }
    for blocks in malformed {
        let mut input = request(vec![]);
        input.regions[0].ir.blocks = blocks;
        let serialized = serde_json::to_vec(&input).unwrap();
        let decoded = serde_json::from_slice(&serialized).unwrap();
        assert!(
            codegen::compile(&decoded).is_err(),
            "accepted malformed blocks: {:?}",
            input.regions[0].ir.blocks
        );
    }
}

#[test]
fn included_static_targets_must_start_blocks_even_for_calls_and_exchanges() {
    for (thumb, target, exchange) in [
        (true, 0x1011, false),
        (true, 0x1011, true),
        (false, 0x1013, false),
        (false, 0x1011, true),
        (false, 0x1010, true),
    ] {
        let size = if thumb { 2 } else { 4 };
        for link in [None, Some(0x1004)] {
            let branch_size = if thumb && link.is_some() { 4 } else { size };
            let mut input = request(vec![]);
            input.regions[0].ir.entry.thumb = thumb;
            input.regions[0].ir.blocks = vec![
                BasicBlock {
                    instructions: vec![Instruction {
                        pc: 0x1000,
                        size: branch_size,
                        condition: Condition::Always,
                        operation: Operation::Branch {
                            target: Value::Immediate(target),
                            link,
                            exchange,
                        },
                    }],
                },
                BasicBlock {
                    instructions: (0..3)
                        .map(|index| Instruction {
                            pc: 0x1010 - (2 - index) * u32::from(size),
                            size,
                            condition: Condition::Always,
                            operation: Operation::Nop,
                        })
                        .collect(),
                },
            ];
            assert!(codegen::compile(&input).is_err(), "thumb={thumb}, link={link:?}, exchange={exchange}");
            let instructions = input.regions[0].ir.blocks[1].instructions.split_off(2);
            input.regions[0].ir.blocks.push(BasicBlock { instructions });
            assert!(codegen::compile(&input).is_ok());
        }
    }
}

#[test]
fn instruction_and_block_caps_accept_the_boundary_and_reject_the_next_item() {
    for thumb in [false, true] {
        let size = if thumb { 2 } else { 4 };
        let instructions: Vec<_> = (0..513)
            .map(|index| Instruction {
                pc: 0x1000 + index * u32::from(size),
                size,
                condition: Condition::Always,
                operation: Operation::Nop,
            })
            .collect();
        let mut input = request(instructions[..512].to_vec());
        input.regions[0].ir.entry.thumb = thumb;
        let check = format!(
            r#"
            for (const start of [0,1,127,255,511]) {{
                reset({{cpsr:{cpsr},pc:0x1000+start*{size},end:0x1000+512*{size},budget:512-start,sample:512-start}});
                assert.equal(entry(F,A),3); assert.equal(get(80),512-start);
                assert.equal(get(60),0x1000+512*{size}); assert.equal(get(72),0); assert.equal(get(76),0);
                assert.equal(samples.length,1);
            }}
            "#,
            cpsr = if thumb { 0x30 } else { 0x10 },
        );
        run(&input, &check);
        input.regions[0].ir.blocks[0].instructions.push(instructions[512].clone());
        assert!(codegen::compile(&input).is_err());

        input.regions[0].ir.blocks = instructions[..512]
            .chunks(4)
            .map(|instructions| BasicBlock {
                instructions: instructions.to_vec(),
            })
            .collect();
        assert_eq!(input.regions[0].ir.blocks.len(), 128);
        run(&input, &check);
        let instructions = input.regions[0].ir.blocks.last_mut().unwrap().instructions.split_off(3);
        input.regions[0].ir.blocks.push(BasicBlock { instructions });
        assert!(codegen::compile(&input).is_err());
    }
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
    let pair = Instruction {
        pc: 0x1000,
        size: 4,
        condition: Condition::Always,
        operation: Operation::Branch {
            target: Value::Immediate(0x2000),
            link: Some(0x1005),
            exchange: false,
        },
    };
    assert!(codegen::compile(&request(vec![pair.clone()])).is_ok());
    assert!(codegen::compile(&request(vec![Instruction { size: 2, ..pair.clone() }])).is_err());
    let mut overlap = request(vec![pair.clone()]);
    overlap.regions[0].ir.blocks.push(BasicBlock {
        instructions: vec![Instruction {
            pc: 0x1002,
            size: 2,
            condition: Condition::Always,
            operation: Operation::Nop,
        }],
    });
    assert!(codegen::compile(&overlap).is_err());
    overlap.regions[0].ir.blocks.reverse();
    assert!(codegen::compile(&overlap).is_err());
    let mut suffix_entry = request(vec![pair.clone()]);
    suffix_entry.regions[0].ir.entry.pc = 0x1002;
    assert!(codegen::compile(&suffix_entry).is_err());
    for operation in [
        Operation::Branch {
            target: Value::Immediate(0x2000),
            link: None,
            exchange: false,
        },
        Operation::Branch {
            target: Value::Register(0),
            link: Some(0x1005),
            exchange: true,
        },
        Operation::Nop,
    ] {
        assert!(codegen::compile(&request(vec![Instruction { operation, ..pair.clone() }])).is_err());
    }
    for (thumb, pc, size) in [(false, 0x1000, 2), (false, 0x1002, 4), (true, 0x1001, 2)] {
        let mut invalid = request(vec![Instruction {
            pc,
            size,
            condition: Condition::Always,
            operation: Operation::Nop,
        }]);
        invalid.regions[0].ir.entry = RegionKey { pc, thumb, cpu_mode: 0x10 };
        assert!(codegen::compile(&invalid).is_err());
    }
}

#[test]
fn malformed_new_operation_operands_are_rejected_at_the_serialized_boundary() {
    let address = Address {
        base: Value::Register(0),
        offset: Operand {
            value: Value::Immediate(0),
            shift: Shift::Lsl,
            amount: ShiftAmount::Immediate(0),
        },
        subtract: false,
        pre_index: true,
        write_back: None,
    };
    let mut operations = vec![
        Operation::MultiplyAccumulate {
            destination: 15,
            left: 0,
            right: 1,
            accumulate: 2,
            set_flags: false,
        },
        Operation::MultiplyAccumulate {
            destination: 0,
            left: 16,
            right: 1,
            accumulate: 2,
            set_flags: false,
        },
        Operation::MultiplyAccumulate {
            destination: 0,
            left: 1,
            right: 16,
            accumulate: 2,
            set_flags: false,
        },
        Operation::MultiplyAccumulate {
            destination: 0,
            left: 1,
            right: 2,
            accumulate: 16,
            set_flags: false,
        },
        Operation::MultiplyLong {
            low: 0,
            high: 0,
            left: 1,
            right: 2,
            signed: false,
            accumulate: false,
            set_flags: false,
        },
        Operation::MultiplyLong {
            low: 15,
            high: 1,
            left: 2,
            right: 3,
            signed: false,
            accumulate: false,
            set_flags: false,
        },
        Operation::MultiplyLong {
            low: 0,
            high: 16,
            left: 2,
            right: 3,
            signed: false,
            accumulate: false,
            set_flags: false,
        },
        Operation::MultiplyLong {
            low: 0,
            high: 1,
            left: 16,
            right: 3,
            signed: false,
            accumulate: false,
            set_flags: false,
        },
        Operation::MultiplyLong {
            low: 0,
            high: 1,
            left: 2,
            right: 16,
            signed: false,
            accumulate: false,
            set_flags: false,
        },
        Operation::ReadStatus { destination: 15 },
        Operation::WriteStatus {
            value: Value::Register(16),
            mask: 0xf000_0000,
        },
        Operation::WriteStatus {
            value: Value::Register(0),
            mask: 0x0800_0000,
        },
        Operation::WriteStatus {
            value: Value::Register(0),
            mask: 0xff,
        },
        Operation::MultipleTransfer {
            base: 15,
            registers: 1,
            increment: true,
            before: false,
            write_back: false,
            load: true,
        },
        Operation::MultipleTransfer {
            base: 0,
            registers: 0,
            increment: true,
            before: false,
            write_back: false,
            load: true,
        },
        Operation::MultipleTransfer {
            base: 0,
            registers: 3,
            increment: true,
            before: false,
            write_back: true,
            load: true,
        },
        Operation::MultipleTransfer {
            base: 1,
            registers: 3,
            increment: true,
            before: false,
            write_back: true,
            load: false,
        },
        Operation::Swap {
            destination: 15,
            address: 0,
            value: 1,
            width: Width::Word,
        },
        Operation::Swap {
            destination: 0,
            address: 15,
            value: 1,
            width: Width::Word,
        },
        Operation::Swap {
            destination: 0,
            address: 1,
            value: 16,
            width: Width::Word,
        },
        Operation::Swap {
            destination: 0,
            address: 1,
            value: 2,
            width: Width::Half,
        },
        Operation::Load {
            destination: 15,
            address,
            width: Width::Byte,
            signed: false,
        },
        Operation::Load {
            destination: 15,
            address,
            width: Width::Word,
            signed: true,
        },
        Operation::Store {
            value: Value::Register(15),
            address,
            width: Width::Half,
        },
        alu(0x1000, AluOp::CountLeadingZeros, Some(0), Value::Immediate(0), Value::Register(1), true).operation,
    ];
    for register in [1, 13, 14, 15, 16, 255] {
        operations.push(Operation::DoubleTransfer {
            register,
            address,
            load: true,
        });
    }
    for bad_address in [
        Address {
            base: Value::Register(16),
            ..address
        },
        Address {
            write_back: Some(15),
            ..address
        },
        Address {
            write_back: Some(2),
            ..address
        },
        Address {
            write_back: Some(3),
            ..address
        },
        Address {
            offset: Operand {
                value: Value::Register(16),
                ..address.offset
            },
            ..address
        },
        Address {
            offset: Operand {
                amount: ShiftAmount::Register(16),
                ..address.offset
            },
            ..address
        },
    ] {
        operations.push(Operation::DoubleTransfer {
            register: 2,
            address: bad_address,
            load: true,
        });
    }
    for operation in operations {
        let input = request(vec![Instruction {
            pc: 0x1000,
            size: 2,
            condition: Condition::Eq,
            operation,
        }]);
        let serialized = serde_json::to_vec(&input).unwrap();
        let decoded: CompileRequest = serde_json::from_slice(&serialized).unwrap();
        assert!(
            codegen::compile(&decoded).is_err(),
            "{:?}",
            decoded.regions[0].ir.blocks[0].instructions[0]
        );
    }
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
    let body = instructions.split_off(1);
    let mut input = request(instructions);
    input.regions[0].ir.blocks.push(BasicBlock { instructions: body });
    run(
        &input,
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
            for (const sample of [1,100]) {
                reset({pc,sample}); const before=Array.from(new Uint8Array(memory.buffer,F,96));
                assert.equal(entry(F,A),0); assert.deepEqual(Array.from(new Uint8Array(memory.buffer,F,96)),before);
                assert.equal(samples.length,0);
            }
        }
    "#,
    );
}
