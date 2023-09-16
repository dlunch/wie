mod framebuffer;
mod image;

use alloc::{vec, vec::Vec};
use core::mem::size_of;

use wie_backend::Image;
use wie_base::util::{read_generic, write_generic};

use crate::{
    base::{CContext, CMemoryId, CMethodBody, CResult},
    method::MethodImpl,
};

use self::{framebuffer::WIPICFramebuffer, image::WIPICImage};

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented graphics{}: {}", id, name)) };

    body.into_body()
}

async fn get_screen_framebuffer(context: &mut dyn CContext, a0: u32) -> CResult<CMemoryId> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({:#x})", a0);

    let framebuffer = WIPICFramebuffer::from_screen_canvas(context)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as u32)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

async fn create_image(context: &mut dyn CContext, ptr_image: u32, image_data: CMemoryId, offset: u32, len: u32) -> CResult<u32> {
    tracing::debug!("MC_grpCreateImage({:#x}, {:#x}, {:#x}, {:#x})", ptr_image, image_data.0, offset, len);

    let image = WIPICImage::new(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<WIPICImage>() as u32)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(0)
}

#[allow(clippy::too_many_arguments)]
async fn draw_image(
    context: &mut dyn CContext,
    framebuffer: CMemoryId,
    dx: u32,
    dy: u32,
    w: u32,
    h: u32,
    image: CMemoryId,
    sx: u32,
    sy: u32,
    graphics_context: u32,
) -> CResult<u32> {
    tracing::debug!(
        "MC_grpDrawImage({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        framebuffer.0,
        dx,
        dy,
        w,
        h,
        image.0,
        sx,
        sy,
        graphics_context
    );

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;
    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;

    let src_image = Image::from_raw(image.width(), image.height(), image.data(context)?);
    let mut canvas = framebuffer.canvas(context)?;

    canvas.draw(dx, dy, w, h, &src_image, sx, sy);

    Ok(0)
}

async fn flush(context: &mut dyn CContext, a0: u32, framebuffer: CMemoryId, a2: u32, a3: u32, a4: u32, a5: u32) -> CResult<u32> {
    tracing::debug!(
        "MC_grpFlushLcd({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        a0,
        framebuffer.0,
        a2,
        a3,
        a4,
        a5
    );

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    let src_canvas = framebuffer.image(context)?;

    let mut screen_canvas = context.backend().screen_canvas();

    screen_canvas.draw(0, 0, framebuffer.width, framebuffer.height, &src_canvas, 0, 0);

    Ok(0)
}

pub fn get_graphics_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0, "MC_grpGetImageProperty"),
        gen_stub(1, "MC_grpGetImageFrameBuffer"),
        get_screen_framebuffer.into_body(),
        gen_stub(3, "MC_grpDestroyOffScreenFrameBuffer"),
        gen_stub(4, "MC_grpCreateOffScreenFrameBuffer"),
        gen_stub(5, "MC_grpInitContext"),
        gen_stub(6, "MC_grpSetContext"),
        gen_stub(7, "MC_grpGetContext"),
        gen_stub(8, "MC_grpPutPixel"),
        gen_stub(9, "MC_grpDrawLine"),
        gen_stub(10, "MC_grpDrawRect"),
        gen_stub(11, "MC_grpFillRect"),
        gen_stub(12, "MC_grpCopyFrameBuffer"),
        draw_image.into_body(),
        gen_stub(14, "MC_grpCopyArea"),
        gen_stub(15, "MC_grpDrawArc"),
        gen_stub(16, "MC_grpFillArc"),
        gen_stub(17, "MC_grpDrawString"),
        gen_stub(18, "MC_grpDrawUnicodeString"),
        gen_stub(19, "MC_grpGetRGBPixels"),
        gen_stub(20, "MC_grpSetRGBPixels"),
        flush.into_body(),
        gen_stub(22, "MC_grpGetPixelFromRGB"),
        gen_stub(23, "MC_grpGetRGBFromPixel"),
        gen_stub(24, "MC_grpGetDisplayInfo"),
        gen_stub(25, "MC_grpRepaint"),
        gen_stub(26, "MC_grpGetFont"),
        gen_stub(27, "MC_grpGetFontHeight"),
        gen_stub(28, "MC_grpGetFontAscent"),
        gen_stub(29, "MC_grpGetFontDescent"),
        gen_stub(30, "MC_grpGetStringWidth"),
        gen_stub(31, "MC_grpGetUnicodeStringWidth"),
        create_image.into_body(),
        gen_stub(33, "MC_grpDestroyImage"),
        gen_stub(34, "MC_grpDecodeNextImage"),
        gen_stub(35, "MC_grpEncodeImage"),
        gen_stub(36, "MC_grpPostEvent"),
        gen_stub(37, "MC_imHandleInput"),
        gen_stub(38, "MC_imSetCurrentMode"),
        gen_stub(39, "MC_imGetCurrentMode"),
        gen_stub(40, "MC_imGetSurpportModeCount"),
        gen_stub(41, "MC_imGetSupportedModes"),
        gen_stub(42, "MC_grpFillPolygon"),
        gen_stub(43, "MC_grpDrawPolygon"),
        gen_stub(44, "OEMC_grpShowAnnunciator"),
        gen_stub(45, "OEMC_grpGetAnnunciatorInfo"),
        gen_stub(46, "OEMC_grpSetAnnunciatorIcon"),
        gen_stub(47, "OEMC_grpGetIdleHelpLineInfo"),
        gen_stub(48, "OEMC_grpShowHelpLine"),
        gen_stub(49, "OEMC_grpGetCharGlyph"),
        gen_stub(50, "OEMC_grpCreateImageEx"),
        gen_stub(51, "OEMC_grpHideHelpLine"),
        gen_stub(52, "OEMC_grpSetCloneScreenFrameBuffer"),
        gen_stub(53, "OEMC_grpGetFontEx"),
        gen_stub(54, "OEMC_grpGetFontLists"),
        gen_stub(55, "OEMC_grpGetFontInfo"),
        gen_stub(56, "OEMC_grpSetFontHelpLine"),
        gen_stub(57, "OEMC_grpGetFontHelpLine"),
        gen_stub(58, "OEMC_grpEncodeImageEx"),
        gen_stub(59, "OEMC_grpGetImageInfo"),
        gen_stub(60, ""),
        gen_stub(61, ""),
        gen_stub(62, ""),
        gen_stub(63, ""),
        gen_stub(64, ""),
        gen_stub(65, ""),
        gen_stub(66, ""),
        gen_stub(67, ""),
        gen_stub(68, ""),
        gen_stub(69, ""),
        gen_stub(70, ""),
        gen_stub(71, ""),
        gen_stub(72, ""),
        gen_stub(73, ""),
        gen_stub(74, ""),
        gen_stub(75, ""),
        gen_stub(76, ""),
        gen_stub(77, ""),
        gen_stub(78, ""),
        gen_stub(79, ""),
    ]
}
