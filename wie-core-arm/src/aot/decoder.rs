use alloc::{vec, vec::Vec};

use wie_arm_jit_types::{CodeImage, CompileRegion, RegionKey};

use super::analysis;

pub(crate) struct Decoder {
    images: vec::IntoIter<CodeImage>,
    image: Option<CodeImage>,
    cursor: usize,
    thumb: bool,
    covered: [u64; 128],
}

impl Decoder {
    pub(crate) fn new(images: Vec<CodeImage>) -> Self {
        Self {
            images: images.into_iter(),
            image: None,
            cursor: 0,
            thumb: false,
            covered: [0; 128],
        }
    }
}

impl Iterator for Decoder {
    type Item = Option<CompileRegion>;

    fn next(&mut self) -> Option<Self::Item> {
        let Some(image) = &self.image else {
            let image = self.images.next()?;
            self.cursor = (4 - image.address as usize % 4) % 4;
            self.thumb = false;
            self.covered.fill(0);
            self.image = Some(image);
            return Some(None);
        };
        for _ in 0..256 {
            if self.cursor >= image.bytes.len() {
                if self.thumb {
                    self.image = None;
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
            return Some(Some(CompileRegion { ir, source }));
        }
        Some(None)
    }
}
