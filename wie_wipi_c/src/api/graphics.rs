mod framebuffer;
mod grp_context;
mod image;

use alloc::{vec, vec::Vec};
use core::mem::size_of;

use bytemuck::Zeroable;

use wie_backend::canvas::{Color, PixelType, Rgb8Pixel};
use wie_common::util::{read_generic, write_generic};

use crate::{
    context::{WIPICContext, WIPICMemoryId, WIPICMethodBody, WIPICResult, WIPICWord},
    method::MethodImpl,
};

use self::{
    framebuffer::{WIPICDisplayInfo, WIPICFramebuffer},
    grp_context::{WIPICGraphicsContext, WIPICGraphicsContextIdx},
    image::WIPICImage,
};

const FRAMEBUFFER_DEPTH: u32 = 16; // XXX hardcode to 16bpp as some game requires 16bpp framebuffer

fn gen_stub(id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented graphics{}: {}", id, name)) };

    body.into_body()
}

async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: WIPICWord) -> WIPICResult<WIPICMemoryId> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({:#x})", a0);

    let (width, height) = {
        let mut platform = context.system().platform();
        let screen = platform.screen();
        (screen.width(), screen.height())
    };

    let framebuffer = WIPICFramebuffer::new(context, width, height, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

async fn init_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord) -> WIPICResult<()> {
    tracing::debug!("MC_grpInitContext({:#x})", p_grp_ctx);

    let grp_ctx: WIPICGraphicsContext = WIPICGraphicsContext::zeroed();
    write_generic(context, p_grp_ctx, grp_ctx)?;
    Ok(())
}

async fn set_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> WIPICResult<()> {
    tracing::debug!("MC_grpSetContext({:#x}, {:?}, {:#x})", p_grp_ctx, op, pv);

    let mut grp_ctx: WIPICGraphicsContext = read_generic(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => {
            grp_ctx.clip = read_generic(context, pv)?;
        }
        WIPICGraphicsContextIdx::FgPixelIdx => {
            grp_ctx.fgpxl = pv;
        }
        WIPICGraphicsContextIdx::BgPixelIdx => {
            grp_ctx.bgpxl = pv;
        }
        WIPICGraphicsContextIdx::TransPixelIdx => {
            grp_ctx.transpxl = pv;
        }
        WIPICGraphicsContextIdx::AlphaIdx => {
            grp_ctx.alpha = pv;
            // grp_ctx.pixel_op_func_ptr = todo!();
            // grp_ctx.param1 = todo!();
        }
        WIPICGraphicsContextIdx::PixelopIdx => {
            grp_ctx.pixel_op_func_ptr = pv;
        }
        WIPICGraphicsContextIdx::PixelParam1Idx => {
            grp_ctx.param1 = pv;
        }
        WIPICGraphicsContextIdx::FontIdx => {
            grp_ctx.font = pv;
        }
        WIPICGraphicsContextIdx::StyleIdx => {
            grp_ctx.style = pv;
        }
        WIPICGraphicsContextIdx::OffsetIdx => {
            grp_ctx.offset = read_generic(context, pv)?;
        }
        _ => {
            tracing::warn!("MC_grpSetContext({:#x}, {:?}, {:#x}): ignoring invalid op", p_grp_ctx, op, pv);
        }
    }
    write_generic(context, p_grp_ctx, grp_ctx)?;

    Ok(())
}

async fn put_pixel(context: &mut dyn WIPICContext, dst_fb: WIPICMemoryId, x: i32, y: i32, p_gctx: WIPICWord) -> WIPICResult<()> {
    tracing::debug!("MC_grpPutPixel({:#x}, {}, {}, {:?})", dst_fb.0, x, y, p_gctx);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst_fb)?)?;
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;

    let mut canvas = framebuffer.canvas(context)?;
    canvas.put_pixel(x as _, y as _, Rgb8Pixel::to_color(gctx.fgpxl));
    Ok(())
}

async fn fill_rect(context: &mut dyn WIPICContext, dst_fb: WIPICMemoryId, x: i32, y: i32, w: i32, h: i32, p_gctx: WIPICWord) -> WIPICResult<()> {
    tracing::debug!("MC_grpFillRect({:#x}, {}, {}, {}, {}, {:#x})", dst_fb.0, x, y, w, h, p_gctx);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst_fb)?)?;
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let mut canvas = framebuffer.canvas(context)?;
    canvas.fill_rect(x as _, y as _, w as _, h as _, Rgb8Pixel::to_color(gctx.fgpxl));
    Ok(())
}

async fn create_image(
    context: &mut dyn WIPICContext,
    ptr_image: WIPICWord,
    image_data: WIPICMemoryId,
    offset: u32,
    len: u32,
) -> WIPICResult<WIPICWord> {
    tracing::debug!("MC_grpCreateImage({:#x}, {:#x}, {}, {})", ptr_image, image_data.0, offset, len);

    let image = WIPICImage::new(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<WIPICImage>() as WIPICWord)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(1) // MC_GRP_IMAGE_DONE
}

#[allow(clippy::too_many_arguments)]
async fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: WIPICMemoryId,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    image: WIPICMemoryId,
    sx: i32,
    sy: i32,
    graphics_context: WIPICWord,
) -> WIPICResult<()> {
    tracing::debug!(
        "MC_grpDrawImage({:#x}, {}, {}, {}, {}, {:#x}, {}, {}, {:#x})",
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

    let src_image = image.img.image(context)?;
    let mut canvas = framebuffer.canvas(context)?;

    canvas.draw(dx as _, dy as _, w as _, h as _, &*src_image, sx as _, sy as _);

    Ok(())
}

async fn flush(
    context: &mut dyn WIPICContext,
    a0: WIPICWord,
    framebuffer: WIPICMemoryId,
    a2: WIPICWord,
    a3: WIPICWord,
    a4: WIPICWord,
    a5: WIPICWord,
) -> WIPICResult<()> {
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

    let mut platform = context.system().platform();
    let screen = platform.screen();

    screen.paint(&*src_canvas);

    Ok(())
}

async fn get_pixel_from_rgb(_context: &mut dyn WIPICContext, r: i32, g: i32, b: i32) -> WIPICResult<WIPICWord> {
    tracing::debug!("MC_grpGetPixelFromRGB({:#x}, {:#x}, {:#x})", r, g, b);
    if (r > 0xff) || (g > 0xff) | (b > 0xff) {
        tracing::debug!("MC_grpGetPixelFromRGB({:#x}, {:#x}, {:#x}): value clipped to 8 bits", r, g, b);
    }

    let color = Rgb8Pixel::from_color(Color {
        a: 0xff,
        r: r as u8,
        g: g as u8,
        b: b as u8,
    }); // TODO do we need to return in screen format?

    Ok(color)
}

async fn get_display_info(context: &mut dyn WIPICContext, reserved: WIPICWord, out_ptr: WIPICWord) -> WIPICResult<WIPICWord> {
    tracing::debug!("MC_grpGetDisplayInfo({:#x}, {:#x})", reserved, out_ptr);

    assert_eq!(reserved, 0);

    let mut platform = context.system().platform();
    let screen = platform.screen();

    let info = WIPICDisplayInfo {
        bpp: FRAMEBUFFER_DEPTH,
        depth: 16,
        width: screen.width(),
        height: screen.height(),
        bpl: 2 * screen.width(),
        color_type: 1, // 1==MC_GRP_DIRECT_COLOR_TYPE
        red_mask: 0xf800,
        green_mask: 0x7e0,
        blue_mask: 0x1f,
    };
    drop(platform);

    write_generic(context, out_ptr, info)?;
    Ok(1)
}

#[allow(clippy::too_many_arguments)]
async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICMemoryId,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    pgc: WIPICWord,
) -> WIPICResult<()> {
    tracing::debug!("MC_grpCopyArea({:#x}, {}, {}, {}, {}, {}, {}, {:#x})", dst.0, dx, dy, w, h, x, y, pgc);

    if w < 0 || h < 0 {
        tracing::warn!("Skipping negative dimension");

        return Ok(());
    }

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst)?)?;

    let image = framebuffer.image(context)?;
    let mut canvas = framebuffer.canvas(context)?;

    canvas.draw(dx as _, dy as _, w as _, h as _, &*image, x as _, y as _);

    Ok(())
}

async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, w: i32, h: i32) -> WIPICResult<WIPICMemoryId> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({}, {})", w, h);

    let framebuffer = WIPICFramebuffer::new(context, w as _, h as _, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

#[allow(clippy::too_many_arguments)]
async fn copy_frame_buffer(
    context: &mut dyn WIPICContext,
    dst: WIPICMemoryId,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    src: WIPICMemoryId,
    sx: i32,
    sy: i32,
    pgc: WIPICWord,
) -> WIPICResult<()> {
    tracing::debug!(
        "MC_grpCopyFrameBuffer({:#x}, {}, {}, {}, {}, {:#x}, {}, {}, {:#x})",
        dst.0,
        dx,
        dy,
        w,
        h,
        src.0,
        sx,
        sy,
        pgc
    );

    let src_framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(src)?)?;
    let dst_framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst)?)?;

    let src_image = src_framebuffer.image(context)?;
    let mut dst_canvas = dst_framebuffer.canvas(context)?;

    dst_canvas.draw(dx as _, dy as _, w as _, h as _, &*src_image, sx as _, sy as _);

    Ok(())
}

pub fn get_graphics_method_table() -> Vec<WIPICMethodBody> {
    vec![
        gen_stub(0, "MC_grpGetImageProperty"),
        gen_stub(1, "MC_grpGetImageFrameBuffer"),
        get_screen_framebuffer.into_body(),
        gen_stub(3, "MC_grpDestroyOffScreenFrameBuffer"),
        create_offscreen_framebuffer.into_body(),
        init_context.into_body(),
        set_context.into_body(),
        gen_stub(7, "MC_grpGetContext"),
        put_pixel.into_body(),
        gen_stub(9, "MC_grpDrawLine"),
        gen_stub(10, "MC_grpDrawRect"),
        fill_rect.into_body(),
        copy_frame_buffer.into_body(),
        draw_image.into_body(),
        copy_area.into_body(),
        gen_stub(15, "MC_grpDrawArc"),
        gen_stub(16, "MC_grpFillArc"),
        gen_stub(17, "MC_grpDrawString"),
        gen_stub(18, "MC_grpDrawUnicodeString"),
        gen_stub(19, "MC_grpGetRGBPixels"),
        gen_stub(20, "MC_grpSetRGBPixels"),
        flush.into_body(),
        get_pixel_from_rgb.into_body(),
        gen_stub(23, "MC_grpGetRGBFromPixel"),
        get_display_info.into_body(),
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
