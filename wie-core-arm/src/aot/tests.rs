use alloc::{boxed::Box, collections::BTreeSet, format, vec, vec::Vec};

use wie_arm_jit_types::{CodeImage, CodePageStamp};
use wie_core_arm_wasm::{Compiler, compile};

use super::decoder::Decoder;

fn image(address: u32, bytes: Vec<u8>) -> CodeImage {
    let end = u64::from(address) + bytes.len() as u64;
    CodeImage {
        address,
        bytes,
        source: (u64::from(address & !0xffff)..end)
            .step_by(0x10000)
            .map(|page| CodePageStamp {
                page: page as u32,
                version: 7,
            })
            .collect(),
    }
}

#[test]
fn raw_images_stream_into_one_module_with_exact_ownership() {
    let images = vec![
        // ARM countdown, unsupported SVC, and a later branch into an owned instruction.
        image(
            0x1000,
            [0xe3a00003_u32, 0xe2500001, 0xe3500000, 0x1afffffc, 0xe12fff1e, 0xef000000, 0xeafffff9]
                .into_iter()
                .flat_map(u32::to_le_bytes)
                .collect(),
        ),
        // Thumb countdown, unsupported SVC, a later owned target, and a truncated BL prefix.
        image(
            0x2000,
            [0x2003_u16, 0x3801, 0x2800, 0xd1fc, 0x4770, 0xdf00, 0xe7f9, 0xf000]
                .into_iter()
                .flat_map(u16::to_le_bytes)
                .collect(),
        ),
        // One region crosses a code-stamp page boundary.
        image(0xfffe, [0x3001_u16, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect()),
    ];
    let expected = compile(Box::new(Decoder::new(images.clone()))).unwrap();
    let mut compiler = Compiler::new(Box::new(Decoder::new(images)));
    let mut steps = 1;
    while !compiler.step().unwrap() {
        steps += 1;
    }
    assert!(steps > expected.manifest.len());
    assert!(compiler.step().unwrap());
    let artifact = compiler.finish();
    assert_eq!(artifact.bytes, expected.bytes);
    assert_eq!(artifact.manifest.len(), expected.manifest.len());
    for (actual, expected) in artifact.manifest.iter().zip(&expected.manifest) {
        assert_eq!(actual.entry, expected.entry);
        assert_eq!(actual.instruction_pcs, expected.instruction_pcs);
        assert_eq!(actual.source, expected.source);
        assert_eq!(actual.export, expected.export);
    }
    assert_eq!(&artifact.bytes[..8], b"\0asm\x01\0\0\0");
    let mut owned = BTreeSet::new();
    for (index, region) in artifact.manifest.iter().enumerate() {
        assert_eq!(region.export, format!("region_{index}"));
        assert_eq!(region.entry.cpu_mode, 0x1f);
        assert!(region.instruction_pcs.contains(&region.entry.pc));
        assert!(region.instruction_pcs.windows(2).all(|pcs| pcs[0] < pcs[1]));
        for &pc in &region.instruction_pcs {
            assert!(owned.insert((region.entry.thumb, pc)), "duplicate ownership at {pc:#x}");
        }
    }
    for (thumb, hole, root) in [(false, 0x1014, 0x1018), (true, 0x200a, 0x200c)] {
        assert!(!owned.contains(&(thumb, hole)));
        let region = artifact
            .manifest
            .iter()
            .find(|region| region.entry.thumb == thumb && region.entry.pc == root)
            .unwrap();
        assert_eq!(region.instruction_pcs, [root]);
    }
    assert!(!owned.contains(&(true, 0x200e)));
    assert_eq!(
        artifact.manifest.iter().find(|region| region.entry.pc == 0xfffe).unwrap().source,
        [CodePageStamp { page: 0, version: 7 }, CodePageStamp { page: 0x10000, version: 7 }]
    );
}

#[test]
fn partial_instructions_remain_fallback_holes() {
    for bytes in [vec![], vec![1], vec![0, 0xf0], vec![0, 0xf0, 0]] {
        assert!(Decoder::new(vec![image(0x1000, bytes)]).flatten().next().is_none());
    }
    let regions: Vec<_> = Decoder::new(vec![image(0x1001, vec![0xff, 1, 0x20, 0])]).flatten().collect();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].ir.entry.pc, 0x1002);
    assert_eq!(regions[0].ir.blocks[0].instructions.len(), 1);
}

#[test]
fn unsupported_images_make_bounded_progress_without_ir() {
    let mut decoder = Decoder::new(vec![image(0x1000, vec![0xff; 16 * 1024])]);
    let mut steps = 0;
    for region in decoder.by_ref() {
        assert!(region.is_none());
        steps += 1;
    }
    assert!(steps >= (16 * 1024 / 4 + 16 * 1024 / 2) / 256);
    assert!(decoder.next().is_none());
}

#[test]
fn analysis_limits_only_claim_emitted_instructions() {
    for code in [vec![0x3001_u16; 513], vec![0xd1ff_u16; 200]] {
        let artifact = compile(Box::new(Decoder::new(vec![image(
            0x1000,
            code.iter().flat_map(|op| op.to_le_bytes()).collect(),
        )])))
        .unwrap();
        let mut pcs = BTreeSet::new();
        let regions: Vec<_> = artifact.manifest.iter().filter(|region| region.entry.thumb).collect();
        assert!(regions.len() > 1);
        for region in regions {
            assert!(region.instruction_pcs.len() <= 512);
            for &pc in &region.instruction_pcs {
                assert!(pcs.insert(pc));
            }
        }
        assert_eq!(pcs, (0..code.len()).map(|index| 0x1000 + index as u32 * 2).collect());
    }
}
