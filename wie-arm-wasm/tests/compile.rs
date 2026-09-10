use wie_arm_jit_types::{
    BasicBlock, CodePageStamp, CompileRegion, CompileRequest, CompiledHandle, Condition, Instruction, Operation, RegionIr, RegionKey,
};
#[path = "../src/codegen.rs"]
mod codegen;

use codegen::compile;

fn request(thumb: bool, count: u32) -> CompileRequest {
    let size = if thumb { 2 } else { 4 };
    CompileRequest {
        session: 11,
        request: 22,
        regions: vec![CompileRegion {
            ir: RegionIr {
                entry: RegionKey {
                    pc: 0x1000,
                    thumb,
                    cpu_mode: 0x10,
                },
                blocks: vec![BasicBlock {
                    instructions: (0..count)
                        .map(|index| Instruction {
                            pc: 0x1000 + index * u32::from(size),
                            size,
                            condition: Condition::Always,
                            operation: Operation::Nop,
                        })
                        .collect(),
                }],
            },
            source: vec![CodePageStamp { page: 0, version: 7 }],
            expected_old: Some(CompiledHandle { slot: 3, generation: 9 }),
        }],
    }
}

#[test]
fn serialized_arm_and_thumb_regions_keep_their_manifest() {
    let mut input = request(false, 2);
    input.regions.extend(request(true, 2).regions);
    let decoded = serde_json::from_slice(&serde_json::to_vec(&input).unwrap()).unwrap();
    let artifact = compile(&decoded).unwrap();
    assert_eq!(&artifact.bytes[..8], b"\0asm\x01\0\0\0");
    assert_eq!(artifact.manifest.len(), input.regions.len());
    for (index, (manifest, region)) in artifact.manifest.iter().zip(&input.regions).enumerate() {
        assert_eq!(manifest.entry, region.ir.entry);
        assert_eq!(manifest.source, region.source);
        assert_eq!(manifest.expected_old, region.expected_old);
        assert_eq!(manifest.export, format!("region_{index}"));
    }
}

#[test]
fn malformed_regions_are_rejected() {
    for thumb in [false, true] {
        for case in 0..6 {
            let mut input = request(thumb, 2);
            let ir = &mut input.regions[0].ir;
            match case {
                0 => ir.entry.cpu_mode = 0x13,
                1 => ir.entry.pc = 0x2000,
                2 => ir.blocks[0].instructions.clear(),
                3 => ir.blocks[0].instructions[0].size = 3,
                4 => ir.blocks[0].instructions[1].pc = 0x1000,
                _ => ir.blocks[0].instructions[1].pc = 0x1010,
            }
            assert!(compile(&input).is_err(), "thumb={thumb}, case={case}");
        }
    }
}

#[test]
fn region_limits_accept_the_boundary_and_reject_overflow() {
    for thumb in [false, true] {
        assert!(compile(&request(thumb, 512)).is_ok());
        assert!(compile(&request(thumb, 513)).is_err());
        let mut input = request(thumb, 129);
        input.regions[0].ir.blocks = input.regions[0].ir.blocks[0]
            .instructions
            .iter()
            .map(|instruction| BasicBlock {
                instructions: vec![instruction.clone()],
            })
            .collect();
        assert!(compile(&input).is_err());
        input.regions[0].ir.blocks.pop();
        assert!(compile(&input).is_ok());
    }
}
extern crate alloc;
