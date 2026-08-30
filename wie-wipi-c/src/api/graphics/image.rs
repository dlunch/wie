use alloc::vec;

use wie_backend::canvas::decode_image;
use wie_util::{Result, WieError};

use wipi_types::wipic::{WIPICImage, WIPICIndirectPtr, WIPICWord};

use crate::{api::graphics::framebuffer::FrameBuffer, context::WIPICContext};

pub fn create_wipi_image(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<WIPICImage> {
    let img_framebuffer = decode_image_framebuffer(context, buf, offset, len)?;
    let mask_framebuffer = FrameBuffer::empty();

    Ok(WIPICImage {
        img: img_framebuffer.0,
        mask: mask_framebuffer.0,
        loop_count: 0,
        delay: 0,
        animated: 0,
        buf,
        offset,
        current: 0,
        len,
    })
}

pub fn decode_image_framebuffer(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<FrameBuffer> {
    let address = context.data_ptr(buf)?.checked_add(offset).ok_or(WieError::AllocationFailure)?;
    let mut data = vec![0; len as usize];
    context.read_bytes(address, &mut data)?;
    let image = decode_image(&data)?;
    FrameBuffer::from_image(context, &*image)
}
