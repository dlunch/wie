use alloc::sync::Arc;

use wie_core_arm_aot::{CodeImage, CompileRegion, RegionKey};

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

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use super::*;

    #[test]
    fn partial_instructions_remain_fallback_holes() {
        for bytes in [vec![], vec![1], vec![0, 0xf0], vec![0, 0xf0, 0]] {
            assert!(
                Decoder::new([CodeImage { address: 0x1000, bytes }].into(), None)
                    .flatten()
                    .next()
                    .is_none()
            );
        }
        let regions: Vec<_> = Decoder::new(
            [CodeImage {
                address: 0x1001,
                bytes: vec![0xff, 1, 0x20, 0],
            }]
            .into(),
            None,
        )
        .flatten()
        .collect();
        assert_eq!(regions.len(), 1);
        assert_eq!(regions[0].ir.entry.pc, 0x1002);
        assert_eq!(regions[0].ir.blocks[0].instructions.len(), 1);
    }

    #[test]
    fn unsupported_images_make_bounded_progress_without_ir() {
        let mut decoder = Decoder::new(
            [CodeImage {
                address: 0x1000,
                bytes: vec![0xff; 16 * 1024],
            }]
            .into(),
            None,
        );
        let mut steps = 0;
        for region in decoder.by_ref() {
            assert!(region.is_none());
            steps += 1;
        }
        assert!(steps >= (16 * 1024 / 4 + 16 * 1024 / 2) / 256);
        assert!(decoder.next().is_none());
    }
}
