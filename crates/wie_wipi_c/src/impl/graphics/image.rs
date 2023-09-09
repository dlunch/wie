use alloc::vec::Vec;

use bytemuck::{Pod, Zeroable};
use wie_backend::Canvas;

use crate::base::{CContext, CMemoryId};

use super::Framebuffer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Image {
    img: Framebuffer,
    mask: Framebuffer,
    loop_count: u32,
    delay: u32,
    animated: u32,
    buf: CMemoryId,
    offset: u32,
    current: u32,
    len: u32,
}

impl Image {
    pub fn new(context: &mut dyn CContext, buf: CMemoryId, offset: u32, len: u32) -> anyhow::Result<Self> {
        let ptr_image_data = context.data_ptr(buf)?;
        let data = context.read_bytes(ptr_image_data + offset, len)?;
        let canvas = Canvas::from_image(&data)?;

        let img_framebuffer = Framebuffer::from_canvas(context, &canvas)?;
        let mask_framebuffer = Framebuffer::empty();

        Ok(Self {
            img: img_framebuffer,
            mask: mask_framebuffer,
            loop_count: 0,
            delay: 0,
            animated: 0,
            buf,
            offset,
            current: 0,
            len,
        })
    }

    pub fn image_width(&self) -> u32 {
        self.img.width
    }

    pub fn image_data(&self, context: &mut dyn CContext) -> anyhow::Result<Vec<u32>> {
        self.img.data(context)
    }
}
