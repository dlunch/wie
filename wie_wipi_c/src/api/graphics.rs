mod framebuffer;
mod grp_context;
mod image;

use core::mem::size_of;

use bytemuck::Zeroable;

use wie_backend::canvas::{Clip, Color, PixelType, Rgb8Pixel};
use wie_util::{Result, read_generic, write_generic};

use crate::{WIPICMemoryId, WIPICWord, context::WIPICContext};

use self::{
    framebuffer::{WIPICDisplayInfo, WIPICFramebuffer},
    grp_context::{WIPICGraphicsContext, WIPICGraphicsContextIdx},
    image::WIPICImage,
};

const FRAMEBUFFER_DEPTH: u32 = 16; // XXX hardcode to 16bpp as some game requires 16bpp framebuffer

pub async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: WIPICWord) -> Result<WIPICMemoryId> {
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

pub async fn init_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpInitContext({:#x})", p_grp_ctx);

    let grp_ctx: WIPICGraphicsContext = WIPICGraphicsContext::zeroed();
    write_generic(context, p_grp_ctx, grp_ctx)?;
    Ok(())
}

pub async fn set_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
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

pub async fn put_pixel(context: &mut dyn WIPICContext, dst_fb: WIPICMemoryId, x: i32, y: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpPutPixel({:#x}, {}, {}, {:?})", dst_fb.0, x, y, p_gctx);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst_fb)?)?;
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;

    let mut canvas = framebuffer.canvas(context)?;
    canvas.put_pixel(x as _, y as _, Rgb8Pixel::to_color(gctx.fgpxl));
    Ok(())
}

pub async fn fill_rect(context: &mut dyn WIPICContext, dst_fb: WIPICMemoryId, x: i32, y: i32, w: i32, h: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpFillRect({:#x}, {}, {}, {}, {}, {:#x})", dst_fb.0, x, y, w, h, p_gctx);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst_fb)?)?;
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let mut canvas = framebuffer.canvas(context)?;

    let clip = Clip {
        x: x as _,
        y: x as _,
        width: w as _,
        height: h as _,
    };

    canvas.fill_rect(x as _, y as _, w as _, h as _, Rgb8Pixel::to_color(gctx.fgpxl), clip);
    Ok(())
}

pub async fn create_image(
    context: &mut dyn WIPICContext,
    ptr_image: WIPICWord,
    image_data: WIPICMemoryId,
    offset: u32,
    len: u32,
) -> Result<WIPICWord> {
    tracing::debug!("MC_grpCreateImage({:#x}, {:#x}, {}, {})", ptr_image, image_data.0, offset, len);

    let image = WIPICImage::new(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<WIPICImage>() as WIPICWord)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(1) // MC_GRP_IMAGE_DONE
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_image(
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
) -> Result<()> {
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

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    canvas.draw(dx as _, dy as _, w as _, h as _, &*src_image, sx as _, sy as _, clip);

    Ok(())
}

pub async fn flush(
    context: &mut dyn WIPICContext,
    a0: WIPICWord,
    framebuffer: WIPICMemoryId,
    a2: WIPICWord,
    a3: WIPICWord,
    a4: WIPICWord,
    a5: WIPICWord,
) -> Result<()> {
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

pub async fn get_pixel_from_rgb(_context: &mut dyn WIPICContext, r: i32, g: i32, b: i32) -> Result<WIPICWord> {
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

pub async fn get_rgb_from_pixel(context: &mut dyn WIPICContext, pixel: i32, r: WIPICWord, g: WIPICWord, b: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_grpGetRGBFromPixel({}, {:#x}, {:#x}, {:#x})", pixel, r, g, b);

    let color = Rgb8Pixel::to_color(pixel as _);

    write_generic(context, r, color.r as i32)?;
    write_generic(context, g, color.g as i32)?;
    write_generic(context, b, color.b as i32)?;

    Ok(pixel)
}

pub async fn get_display_info(context: &mut dyn WIPICContext, reserved: WIPICWord, out_ptr: WIPICWord) -> Result<WIPICWord> {
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
pub async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICMemoryId,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpCopyArea({:#x}, {}, {}, {}, {}, {}, {}, {:#x})", dst.0, dx, dy, w, h, x, y, pgc);

    if w < 0 || h < 0 {
        tracing::warn!("Skipping negative dimension");

        return Ok(());
    }

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(dst)?)?;

    let image = framebuffer.image(context)?;
    let mut canvas = framebuffer.canvas(context)?;

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    canvas.draw(dx as _, dy as _, w as _, h as _, &*image, x as _, y as _, clip);

    Ok(())
}

pub async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, w: i32, h: i32) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({}, {})", w, h);

    let framebuffer = WIPICFramebuffer::new(context, w as _, h as _, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

pub async fn destroy_offscreen_framebuffer(context: &mut dyn WIPICContext, framebuffer: WIPICMemoryId) -> Result<()> {
    tracing::debug!("MC_grpDestroyOffScreenFrameBuffer({:#x})", framebuffer.0);

    context.free(framebuffer)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_frame_buffer(
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
) -> Result<()> {
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

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    dst_canvas.draw(dx as _, dy as _, w as _, h as _, &*src_image, sx as _, sy as _, clip);

    Ok(())
}

pub async fn get_font(_: &mut dyn WIPICContext, face: i32, size: i32, style: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFont({}, {}, {})", face, size, style);

    Ok(0)
}

pub async fn get_font_height(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontHeight({})", font);

    Ok(10)
}

pub async fn get_string_width(_: &mut dyn WIPICContext, font: i32, ptr_string: WIPICWord, length: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetStringWidth({}, {:#x}, {})", font, ptr_string, length);

    Ok(10)
}

pub async fn draw_string(_: &mut dyn WIPICContext, dst: WIPICMemoryId, x: i32, y: i32, string: WIPICWord, length: i32, pgc: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_grpDrawString({:#x}, {}, {}, {:#x}, {}, {:#x})", dst.0, x, y, string, length, pgc);

    Ok(())
}

pub async fn repaint(context: &mut dyn WIPICContext, lcd: i32, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    tracing::warn!("stub MC_grpRepaint({}, {}, {}, {}, {})", lcd, x, y, width, height);

    let mut platform = context.system().platform();
    let screen = platform.screen();
    screen.request_redraw().unwrap();

    Ok(())
}

pub async fn get_image_property(context: &mut dyn WIPICContext, image: WIPICMemoryId, property: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetImageProperty({:#x}, {})", image.0, property);

    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;

    Ok(match property {
        4 => image.img.width as _,
        5 => image.img.height as _,
        _ => {
            tracing::warn!("unknown property {}", property);
            0
        }
    })
}

// it's not documented api, but lgt apps gets pointer via api call
pub async fn get_framebuffer_pointer(context: &mut dyn WIPICContext, framebuffer: WIPICMemoryId) -> Result<WIPICWord> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_POINTER({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.buf.0)
}

pub async fn get_framebuffer_width(context: &mut dyn WIPICContext, framebuffer: WIPICMemoryId) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_WIDTH({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.width as _)
}

pub async fn get_framebuffer_height(context: &mut dyn WIPICContext, framebuffer: WIPICMemoryId) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_HEIGHT({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.height as _)
}
