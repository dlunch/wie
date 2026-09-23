use alloc::sync::Arc;

use wie_arm_jit_types::{CodeImage, CompileRegion, RegionKey};

use super::analysis;

pub(crate) struct Decoder {
    images: Arc<[CodeImage]>,
    image_index: usize,
    image_ready: bool,
    cursor: usize,
    thumb: bool,
    mode: Option<bool>,
    covered: [u64; 128],
}

impl Decoder {
    pub(crate) fn new(images: Arc<[CodeImage]>, mode: Option<bool>) -> Self {
        Self {
            images,
            image_index: 0,
            image_ready: false,
            cursor: 0,
            thumb: false,
            mode,
            covered: [0; 128],
        }
    }
}

impl Iterator for Decoder {
    type Item = Option<CompileRegion>;

    fn next(&mut self) -> Option<Self::Item> {
        let image = self.images.get(self.image_index)?;
        if !self.image_ready {
            self.thumb = self.mode.unwrap_or(false);
            let alignment = if self.thumb { 2 } else { 4 };
            self.cursor = (alignment - image.address as usize % alignment) % alignment;
            self.covered.fill(0);
            self.image_ready = true;
            return Some(None);
        }
        for _ in 0..256 {
            if self.cursor >= image.bytes.len() {
                if self.thumb || self.mode.is_some() {
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
                let offset = (instruction.pc.get() - image.address) as usize;
                self.covered[offset / 128] |= 1 << ((offset / 2) % 64);
            }
            return Some(Some(CompileRegion { ir }));
        }
        Some(None)
    }
}
