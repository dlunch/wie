use alloc::sync::Arc;

use wie_arm_jit_types::{CodeImage, CompileRegion, RegionKey};

use super::analysis;

pub(crate) struct Decoder {
    images: Arc<[CodeImage]>,
    image_index: usize,
    image_ready: bool,
    cursor: usize,
    thumb: bool,
    covered: [u64; 128],
}

impl Decoder {
    pub(crate) fn new(images: Arc<[CodeImage]>) -> Self {
        Self {
            images,
            image_index: 0,
            image_ready: false,
            cursor: 0,
            thumb: false,
            covered: [0; 128],
        }
    }
}

impl Iterator for Decoder {
    type Item = Option<CompileRegion>;

    fn next(&mut self) -> Option<Self::Item> {
        let image = self.images.get(self.image_index)?;
        if !self.image_ready {
            self.cursor = (4 - image.address as usize % 4) % 4;
            self.thumb = false;
            self.covered.fill(0);
            self.image_ready = true;
            return Some(None);
        }
        for _ in 0..256 {
            if self.cursor >= image.bytes.len() {
                if self.thumb {
                    self.image_index += 1;
                    self.image_ready = false;
                } else {
                    self.thumb = true;
                    self.cursor = (2 - image.address as usize % 2) % 2;
                    self.covered.fill(0);
                }
                return Some(None);
            }
            let offset = self.cursor;
            self.cursor += if self.thumb { 2 } else { 4 };
            if self.covered[offset / 128] & (1 << ((offset / 2) % 64)) != 0 {
                continue;
            }
            let entry = RegionKey {
                pc: image.address + offset as u32,
                thumb: self.thumb,
                cpu_mode: 0x1f,
            };
            let Some(ir) = analysis::analyze(&image.bytes, image.address, entry, &self.covered) else {
                continue;
            };
            for instruction in ir.blocks.iter().flat_map(|block| &block.instructions) {
                let offset = (instruction.pc - image.address) as usize;
                self.covered[offset / 128] |= 1 << ((offset / 2) % 64);
            }
            let source = image
                .source
                .iter()
                .filter(|stamp| {
                    ir.blocks.iter().flat_map(|block| &block.instructions).any(|instruction| {
                        stamp.page >= (instruction.pc & !0xffff)
                            && u64::from(stamp.page) <= ((u64::from(instruction.pc) + u64::from(instruction.size) - 1) & !0xffff)
                    })
                })
                .copied()
                .collect();
            let source_bytes = ir
                .blocks
                .iter()
                .map(|block| {
                    let first = block.instructions[0].pc;
                    let last = block.instructions.last().unwrap();
                    let start = (first - image.address) as usize;
                    let end = (last.pc - image.address) as usize + usize::from(last.size);
                    (first, image.bytes[start..end].to_vec())
                })
                .collect();
            return Some(Some(CompileRegion { ir, source, source_bytes }));
        }
        Some(None)
    }
}
