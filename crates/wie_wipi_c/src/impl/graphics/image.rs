use alloc::vec::Vec;

use bytemuck::{Pod, Zeroable};

use wie_backend::Image;

use crate::base::{CContext, CMemoryId};

use super::WIPICFramebuffer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICImage {
    img: WIPICFramebuffer,
    mask: WIPICFramebuffer,
    loop_count: u32,
    delay: u32,
    animated: u32,
    buf: CMemoryId,
    offset: u32,
    current: u32,
    len: u32,
}

impl WIPICImage {
    pub fn new(context: &mut dyn CContext, buf: CMemoryId, offset: u32, len: u32) -> anyhow::Result<Self> {
        let ptr_image_data = context.data_ptr(buf)?;
        let data = context.read_bytes(ptr_image_data + offset, len)?;
        let image = Image::from_image(&data)?;

        let img_framebuffer = WIPICFramebuffer::from_image(context, &image)?;
        let mask_framebuffer = WIPICFramebuffer::empty();

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

    pub fn width(&self) -> u32 {
        self.img.width
    }

    pub fn height(&self) -> u32 {
        self.img.height
    }

    pub fn data(&self, context: &mut dyn CContext) -> anyhow::Result<Vec<u8>> {
        self.img.data(context)
    }
}
