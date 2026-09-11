use alloc::{boxed::Box, collections::BTreeSet, sync::Arc, vec, vec::Vec};
use core::sync::atomic::{AtomicUsize, Ordering};

use wie_arm_jit_types::{CodeImage, CodePageStamp, CompileRequest};
use wie_core_arm_wasm::{Compiler, bind_manifest_source, compile};

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
fn shared_inputs_can_be_inspected_without_consuming_the_decoder() {
    let images: Arc<[_]> = vec![image(0x1000, vec![1, 0x30, 0x70, 0x47])].into();
    let steps = Arc::new(AtomicUsize::new(0));
    let observed = steps.clone();
    let request = CompileRequest {
        images: images.clone(),
        regions: Box::new(Decoder::new(images.clone()).inspect(move |_| {
            observed.fetch_add(1, Ordering::Relaxed);
        })),
    };
    assert!(Arc::ptr_eq(&request.images, &images));
    assert_eq!(request.images[0].bytes, [1, 0x30, 0x70, 0x47]);
    assert_eq!(steps.load(Ordering::Relaxed), 0);
    let artifact = compile(request).unwrap();
    assert!(steps.load(Ordering::Relaxed) > 0);
    assert!(artifact.manifest.iter().any(|region| region.entry.thumb));
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
        image(0x20000, [0xf000_u16, 0xf800, 0x4770].into_iter().flat_map(u16::to_le_bytes).collect()),
    ];
    let images: Arc<[_]> = images.into();
    let expected = compile(CompileRequest {
        images: images.clone(),
        regions: Box::new(Decoder::new(images.clone())),
    })
    .unwrap();
    let mut compiler = Compiler::new(CompileRequest {
        images: images.clone(),
        regions: Box::new(Decoder::new(images.clone())),
    });
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
        assert_eq!(actual.source_bytes, expected.source_bytes);
        let mut restored = actual.clone();
        restored.source.clear();
        bind_manifest_source(&mut restored, &images).unwrap();
        assert_eq!(restored.source, actual.source);
    }
    assert_eq!(&artifact.bytes[..8], b"\0asm\x01\0\0\0");
    let mut owned = BTreeSet::new();
    for region in &artifact.manifest {
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
    assert_eq!(
        artifact.manifest.iter().find(|region| region.entry.pc == 0xfffe).unwrap().source_bytes,
        [(0xfffe, vec![1, 0x30, 0x70, 0x47])]
    );
}

#[test]
fn partial_instructions_remain_fallback_holes() {
    for bytes in [vec![], vec![1], vec![0, 0xf0], vec![0, 0xf0, 0]] {
        assert!(Decoder::new(vec![image(0x1000, bytes)].into()).flatten().next().is_none());
    }
    let regions: Vec<_> = Decoder::new(vec![image(0x1001, vec![0xff, 1, 0x20, 0])].into()).flatten().collect();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].ir.entry.pc, 0x1002);
    assert_eq!(regions[0].ir.blocks[0].instructions.len(), 1);
}

#[test]
fn unsupported_images_make_bounded_progress_without_ir() {
    let mut decoder = Decoder::new(vec![image(0x1000, vec![0xff; 16 * 1024])].into());
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
        let images: Arc<[_]> = vec![image(0x1000, code.iter().flat_map(|op| op.to_le_bytes()).collect())].into();
        let artifact = compile(CompileRequest {
            images: images.clone(),
            regions: Box::new(Decoder::new(images)),
        })
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
