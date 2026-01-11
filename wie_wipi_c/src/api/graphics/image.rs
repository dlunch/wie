use alloc::vec;

use wie_backend::canvas::decode_image;
use wie_util::Result;

use wipi_types::wipic::{WIPICImage, WIPICIndirectPtr, WIPICWord};

use crate::{api::graphics::framebuffer::FrameBuffer, context::WIPICContext};

pub fn create_wipi_image(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<WIPICImage> {
    let ptr_image_data = context.data_ptr(buf)?;

    let mut data = vec![0; len as _];
    context.read_bytes(ptr_image_data + offset, &mut data)?;
    let image = decode_image(&data)?;

    let img_framebuffer = FrameBuffer::from_image(context, &*image)?;
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
