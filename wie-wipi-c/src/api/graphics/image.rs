use alloc::{boxed::Box, vec, vec::Vec};

use wie_backend::canvas::{ArgbPixel, Image, PixelType, Rgb565Pixel, VecImageBuffer, decode_image};
use wie_util::{Result, WieError};

use wipi_types::wipic::{WIPICImage, WIPICIndirectPtr, WIPICWord};

use crate::{api::graphics::framebuffer::FrameBuffer, context::WIPICContext};

pub fn create_wipi_image(context: &mut dyn WIPICContext, buf: WIPICIndirectPtr, offset: WIPICWord, len: WIPICWord) -> Result<WIPICImage> {
    let address = context.data_ptr(buf)?.checked_add(offset).ok_or(WieError::AllocationFailure)?;
    let mut data = vec![0; len as usize];
    context.read_bytes(address, &mut data)?;
    let image = decode_image(&data)?;
    let colors = image.colors();
    // Native callers read image pixels in the display's RGB565 format.
    let pixels = VecImageBuffer::<Rgb565Pixel>::from_raw(
        image.width(),
        image.height(),
        colors.iter().copied().map(Rgb565Pixel::from_color).collect(),
    );
    let img_framebuffer = FrameBuffer::from_image(context, &pixels)?;
    let mask_framebuffer = if colors.iter().any(|color| color.a != 255) {
        let mask = FrameBuffer::new(context, image.width(), image.height(), 8)?;
        mask.write(context, &colors.iter().map(|color| color.a).collect::<Vec<_>>())?;
        mask
    } else {
        FrameBuffer::empty()
    };

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

pub fn image_buffer(context: &mut dyn WIPICContext, image: &WIPICImage) -> Result<Box<dyn Image>> {
    let pixels = FrameBuffer(image.img).image(context)?;
    if image.mask.buf.0 == 0 {
        return Ok(pixels);
    }

    let mut alpha = vec![0; (image.mask.bpl * image.mask.height) as usize];
    context.read_bytes(context.data_ptr(image.mask.buf)?, &mut alpha)?;
    let colors = pixels
        .colors()
        .into_iter()
        .zip(alpha)
        .map(|(mut color, alpha)| {
            color.a = alpha;
            ArgbPixel::from_color(color)
        })
        .collect();
    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(image.img.width, image.img.height, colors)))
}
