use alloc::vec;

use bytemuck::{Pod, Zeroable};

use wie_backend::canvas::decode_image;
use wie_util::Result;

use crate::{WIPICMemoryId, WIPICWord, context::WIPICContext};

use super::WIPICFramebuffer;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICImage {
    pub img: WIPICFramebuffer,
    pub mask: WIPICFramebuffer,
    pub loop_count: WIPICWord,
    pub delay: WIPICWord,
    pub animated: WIPICWord,
    pub buf: WIPICMemoryId,
    pub offset: WIPICWord,
    pub current: WIPICWord,
    pub len: WIPICWord,
}

impl WIPICImage {
    pub fn new(context: &mut dyn WIPICContext, buf: WIPICMemoryId, offset: WIPICWord, len: WIPICWord) -> Result<Self> {
        let ptr_image_data = context.data_ptr(buf)?;

        let mut data = vec![0; len as _];
        context.read_bytes(ptr_image_data + offset, &mut data)?;
        let image = decode_image(&data)?;

        let img_framebuffer = WIPICFramebuffer::from_image(context, &*image)?;
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
}
