mod framebuffer;
mod grp_context;
mod image;
pub mod primitives;

pub use framebuffer::FrameBuffer;
pub use image::decode_image_framebuffer;

use core::mem::size_of;

use wie_backend::{
    Event,
    canvas::{ArgbPixel, Clip, Color, FONT_SIZE, ImageBuffer, PixelType, Rgb565Pixel, VecImageBuffer, encode_bmp, string_width},
};
use wie_util::{Result, read_generic, write_generic};

use wipi_types::wipic::{WIPICDisplayInfo, WIPICFramebuffer, WIPICGraphicsContext, WIPICImage, WIPICIndirectPtr, WIPICWord};

use crate::{context::WIPICContext, globals::SCREEN_FRAMEBUFFER_PTR};

use self::{grp_context::WIPICGraphicsContextIdx, image::create_wipi_image};

const FRAMEBUFFER_DEPTH: u32 = 16; // XXX hardcode to 16bpp as some apps require a 16bpp framebuffer

pub async fn fill_polygon(
    _context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x_points: WIPICWord,
    y_points: WIPICWord,
    n_points: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::warn!("stub MC_grpFillPolygon({:#x}, {x_points:#x}, {y_points:#x}, {n_points}, {pgc:#x})", dst.0);

    Ok(())
}

pub async fn get_screen_framebuffer(context: &mut dyn WIPICContext, a0: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetScreenFrameBuffer({a0:#x})");

    let framebuffer_ptr: u32 = read_generic(context, SCREEN_FRAMEBUFFER_PTR)?;
    if framebuffer_ptr != 0 {
        return Ok(WIPICIndirectPtr(framebuffer_ptr));
    }

    let (width, height) = {
        let platform = context.system().platform();
        let screen = platform.screen();
        (screen.width(), screen.height())
    };

    let framebuffer = FrameBuffer::new(context, width, height, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;
    write_generic(context, SCREEN_FRAMEBUFFER_PTR, memory.0)?;

    Ok(memory)
}

pub async fn init_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpInitContext({p_grp_ctx:#x})");

    let grp_ctx = WIPICGraphicsContext::default();
    write_generic(context, p_grp_ctx, grp_ctx)?;
    Ok(())
}

pub async fn set_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let mut grp_ctx: WIPICGraphicsContext = read_generic(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => {
            // MC_GRP_CONTEXT_CLIP_MASK is cleared when clipping is disabled.
            if pv == 0 {
                grp_ctx.mask &= !1;
            } else {
                let clip: [i32; 4] = read_generic(context, pv)?;
                grp_ctx.clip = clip.map(|value| value as u16);
                grp_ctx.mask |= 1;
            }
        }
        WIPICGraphicsContextIdx::FgPixelIdx => {
            grp_ctx.fgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::BgPixelIdx => {
            grp_ctx.bgpxl = pv as _;
        }
        WIPICGraphicsContextIdx::TransPixelIdx => {
            grp_ctx.transpxl = pv as _;
        }
        WIPICGraphicsContextIdx::AlphaIdx => {
            grp_ctx.alpha = pv as _;
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
            let offset: [i32; 2] = read_generic(context, pv)?;
            grp_ctx.offset = offset.map(|value| value as u16);
        }
        _ => {
            tracing::warn!("MC_grpSetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op");
        }
    }
    write_generic(context, p_grp_ctx, grp_ctx)?;

    Ok(())
}

pub async fn get_context(context: &mut dyn WIPICContext, p_grp_ctx: WIPICWord, op: WIPICGraphicsContextIdx, pv: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x})");

    let grp_ctx: WIPICGraphicsContext = read_generic(context, p_grp_ctx)?;
    match op {
        WIPICGraphicsContextIdx::ClipIdx => write_generic(context, pv, grp_ctx.clip.map(|value| i32::from(value as i16)))?,
        WIPICGraphicsContextIdx::FgPixelIdx => write_generic(context, pv, grp_ctx.fgpxl)?,
        WIPICGraphicsContextIdx::BgPixelIdx => write_generic(context, pv, grp_ctx.bgpxl)?,
        WIPICGraphicsContextIdx::TransPixelIdx => write_generic(context, pv, grp_ctx.transpxl)?,
        WIPICGraphicsContextIdx::AlphaIdx => write_generic(context, pv, grp_ctx.alpha)?,
        WIPICGraphicsContextIdx::PixelopIdx => write_generic(context, pv, grp_ctx.pixel_op_func_ptr)?,
        WIPICGraphicsContextIdx::PixelParam1Idx => write_generic(context, pv, grp_ctx.param1)?,
        WIPICGraphicsContextIdx::FontIdx => write_generic(context, pv, grp_ctx.font)?,
        WIPICGraphicsContextIdx::StyleIdx => write_generic(context, pv, grp_ctx.style)?,
        WIPICGraphicsContextIdx::OffsetIdx => write_generic(context, pv, grp_ctx.offset.map(|value| i32::from(value as i16)))?,
        _ => tracing::warn!("MC_grpGetContext({p_grp_ctx:#x}, {op:?}, {pv:#x}): ignoring invalid op"),
    }

    Ok(())
}

pub async fn put_pixel(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpPutPixel({:#x}, {x}, {y}, {p_gctx:?})", dst_fb.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::put_pixel(
        context,
        &framebuffer,
        x as _,
        y as _,
        color,
        Clip {
            x: 0,
            y: 0,
            width: framebuffer.0.width,
            height: framebuffer.0.height,
        },
    )
}

pub async fn fill_rect(context: &mut dyn WIPICContext, dst_fb: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, p_gctx: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpFillRect({:#x}, {x}, {y}, {w}, {h}, {p_gctx:#x})", dst_fb.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst_fb)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::fill_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip)
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_arc(context, &framebuffer, x, y, w as u32, h as u32, start_angle, arc_angle, color, clip)
}

#[allow(clippy::too_many_arguments)]
pub async fn fill_arc(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    start_angle: i32,
    arc_angle: i32,
    p_gctx: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFillArc({:#x}, {x}, {y}, {w}, {h}, {start_angle}, {arc_angle}, {p_gctx:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, p_gctx)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::fill_arc(context, &framebuffer, x, y, w as u32, h as u32, start_angle, arc_angle, color, clip)
}

pub async fn create_image(
    context: &mut dyn WIPICContext,
    ptr_image: WIPICWord,
    image_data: WIPICIndirectPtr,
    offset: u32,
    len: u32,
) -> Result<WIPICWord> {
    tracing::debug!("MC_grpCreateImage({ptr_image:#x}, {:#x}, {offset}, {len})", image_data.0);

    let image = create_wipi_image(context, image_data, offset, len)?;

    let memory = context.alloc(size_of::<WIPICImage>() as WIPICWord)?;
    write_generic(context, ptr_image, memory)?;
    write_generic(context, context.data_ptr(memory)?, image)?;

    Ok(1) // MC_GRP_IMAGE_DONE
}

pub async fn encode_image(
    context: &mut dyn WIPICContext,
    source: WIPICIndirectPtr,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    length: WIPICWord,
) -> Result<u32> {
    tracing::debug!("MC_grpEncodeImage({:#x}, {x}, {y}, {width}, {height}, {length:#x})", source.0);
    write_generic(context, length, 0u32)?;
    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(source)?)?);
    if x < 0
        || y < 0
        || width <= 0
        || height <= 0
        || i64::from(x) + i64::from(width) > i64::from(framebuffer.0.width)
        || i64::from(y) + i64::from(height) > i64::from(framebuffer.0.height)
    {
        return Ok(0);
    }

    let source = framebuffer.image(context)?;
    let mut region = VecImageBuffer::<ArgbPixel>::new(width as u32, height as u32);
    for row in 0..height {
        for col in 0..width {
            region.put_pixel(col, row, source.get_pixel(x + col, y + row));
        }
    }
    let encoded = encode_bmp(&region)?;
    let buffer = context.alloc(encoded.len() as u32)?;
    context.write_bytes(context.data_ptr(buffer)?, &encoded)?;
    write_generic(context, length, encoded.len() as u32)?;
    Ok(buffer.0)
}

pub async fn destroy_image(context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyImage({:#x})", image.0);

    context.free(image)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn draw_image(
    context: &mut dyn WIPICContext,
    framebuffer: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    image: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    graphics_context: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpDrawImage({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {graphics_context:#x})",
        framebuffer.0,
        image.0
    );

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);
    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;
    let gctx: WIPICGraphicsContext = read_generic(context, graphics_context)?;

    let src_image = FrameBuffer(image.img).image(context)?;
    let clip = if gctx.mask & 1 != 0 {
        let [left, top, right, bottom] = gctx.clip.map(|value| i32::from(value as i16));
        Clip {
            x: left,
            y: top,
            width: (right - left).max(0) as u32,
            height: (bottom - top).max(0) as u32,
        }
    } else {
        Clip {
            x: dx,
            y: dy,
            width: w as u32,
            height: h as u32,
        }
    };

    primitives::draw_image(context, &framebuffer, dx, dy, w as u32, h as u32, &*src_image, sx, sy, clip)
}

pub async fn flush_lcd(
    context: &mut dyn WIPICContext,
    i: WIPICWord,
    framebuffer: WIPICIndirectPtr,
    x: WIPICWord,
    y: WIPICWord,
    w: WIPICWord,
    h: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpFlushLcd({i:#x}, {:#x}, {x:#x}, {y:#x}, {w:#x}, {h:#x})", framebuffer.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(framebuffer)?)?);

    let src_canvas = framebuffer.image(context)?;

    let platform = context.system().platform();
    let screen = platform.screen();

    screen.paint(&*src_canvas);

    Ok(())
}

pub async fn get_pixel_from_rgb(_context: &mut dyn WIPICContext, r: i32, g: i32, b: i32) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x})");
    if (r > 0xff) || (g > 0xff) | (b > 0xff) {
        tracing::debug!("MC_grpGetPixelFromRGB({r:#x}, {g:#x}, {b:#x}): value clipped to 8 bits");
    }

    let color = Rgb565Pixel::from_color(Color {
        a: 0xff,
        r: r as u8,
        g: g as u8,
        b: b as u8,
    });

    Ok(color as WIPICWord)
}

pub async fn get_rgb_from_pixel(context: &mut dyn WIPICContext, pixel: i32, r: WIPICWord, g: WIPICWord, b: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_grpGetRGBFromPixel({pixel}, {r:#x}, {g:#x}, {b:#x})");

    let color = Rgb565Pixel::to_color(pixel as u16);

    write_generic(context, r, color.r as i32)?;
    write_generic(context, g, color.g as i32)?;
    write_generic(context, b, color.b as i32)?;

    Ok(pixel)
}

pub async fn get_display_info(context: &mut dyn WIPICContext, reserved: WIPICWord, out_ptr: WIPICWord) -> Result<WIPICWord> {
    tracing::debug!("MC_grpGetDisplayInfo({reserved:#x}, {out_ptr:#x})");

    assert_eq!(reserved, 0);

    let platform = context.system().platform();
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

    write_generic(context, out_ptr, info)?;
    Ok(1)
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_area(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    x: i32,
    y: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpCopyArea({:#x}, {dx}, {dy}, {w}, {h}, {x}, {y}, {pgc:#x})", dst.0);

    if w < 0 || h < 0 {
        tracing::warn!("Skipping negative dimension");

        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    primitives::copy_area(context, &framebuffer, dx, dy, w as u32, h as u32, x, y, clip)
}

pub async fn create_offscreen_framebuffer(context: &mut dyn WIPICContext, w: i32, h: i32) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpCreateOffScreenFrameBuffer({w}, {h})");

    let framebuffer = FrameBuffer::new(context, w as _, h as _, FRAMEBUFFER_DEPTH)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as WIPICWord)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer.0)?;

    Ok(memory)
}

pub async fn destroy_offscreen_framebuffer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<()> {
    tracing::debug!("MC_grpDestroyOffScreenFrameBuffer({:#x})", framebuffer.0);

    context.free(framebuffer)?;

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn copy_frame_buffer(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    dx: i32,
    dy: i32,
    w: i32,
    h: i32,
    src: WIPICIndirectPtr,
    sx: i32,
    sy: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!(
        "MC_grpCopyFrameBuffer({:#x}, {dx}, {dy}, {w}, {h}, {:#x}, {sx}, {sy}, {pgc:#x})",
        dst.0,
        src.0
    );

    let src_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let dst_framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);

    let clip = Clip {
        x: dx as _,
        y: dy as _,
        width: w as _,
        height: h as _,
    };

    primitives::copy_framebuffer(context, &dst_framebuffer, dx, dy, w as u32, h as u32, &src_framebuffer, sx, sy, clip)
}

pub async fn get_font(_: &mut dyn WIPICContext, face: i32, size: i32, style: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFont({face}, {size}, {style})");

    Ok(0)
}

pub async fn get_font_height(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontHeight({font})");

    Ok(12)
}

pub async fn get_font_ascent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontAscent({font})");

    Ok(10)
}

pub async fn get_font_descent(_: &mut dyn WIPICContext, font: i32) -> Result<i32> {
    tracing::warn!("stub MC_grpGetFontDescent({font})");

    Ok(2)
}

pub async fn get_string_width(context: &mut dyn WIPICContext, font: i32, ptr_string: WIPICWord, length: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetStringWidth({font}, {ptr_string:#x}, {length})");

    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(0);
    };
    Ok(string_width(context.system().platform().font(), &string, FONT_SIZE) as i32)
}

pub async fn draw_string(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    ptr_string: WIPICWord,
    length: i32,
    pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpDrawString({:#x}, {x}, {y}, {ptr_string:#x}, {length}, {pgc:#x})", dst.0);

    let Some(string) = primitives::read_text(context, ptr_string, length)? else {
        return Ok(());
    };

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;

    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_text(context, &framebuffer, &string, x, y, color, clip)
}

pub async fn repaint(context: &mut dyn WIPICContext, lcd: i32, x: i32, y: i32, width: i32, height: i32) -> Result<()> {
    tracing::debug!("MC_grpRepaint({lcd}, {x}, {y}, {width}, {height})");

    context.system().event_queue().push(Event::Repaint);

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn get_rgb_pixels(
    context: &mut dyn WIPICContext,
    src: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    pd: WIPICWord,
    ipl: i32,
) -> Result<()> {
    tracing::debug!("MC_grpGetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {pd:#x}, {ipl})", src.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }
    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(src)?)?);
    let image = framebuffer.image(context)?;
    primitives::get_rgb_pixels(context, &*image, x, y, w, h, pd, ipl)
}

#[allow(clippy::too_many_arguments)]
pub async fn set_rgb_pixels(
    context: &mut dyn WIPICContext,
    dst: WIPICIndirectPtr,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    psrc: WIPICWord,
    ibpl: i32,
    _pgc: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_grpSetRGBPixels({:#x}, {x}, {y}, {w}, {h}, {psrc:#x}, {ibpl})", dst.0);
    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width,
        height: framebuffer.0.height,
    };
    primitives::set_rgb_pixels(context, &framebuffer, x, y, w, h, psrc, ibpl, clip)
}

pub async fn get_image_framebuffer(_context: &mut dyn WIPICContext, image: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_grpGetImageFrameBuffer({:#x})", image.0);

    // WIPICImage starts with `img: WIPICFramebuffer` at offset 0,
    // so the image handle doubles as a framebuffer handle.
    Ok(image)
}

pub async fn get_image_property(context: &mut dyn WIPICContext, image: WIPICIndirectPtr, property: i32) -> Result<i32> {
    tracing::debug!("MC_grpGetImageProperty({:#x}, {property})", image.0);

    let image: WIPICImage = read_generic(context, context.data_ptr(image)?)?;

    Ok(match property {
        4 => image.img.width as _,
        5 => image.img.height as _,
        _ => {
            tracing::warn!("unknown property {property}");
            0
        }
    })
}

pub async fn draw_rect(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x: i32, y: i32, w: i32, h: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawRect({:#x}, {x}, {y}, {w}, {h}, {pgc:#x})", dst.0);

    if w <= 0 || h <= 0 {
        return Ok(());
    }

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;
    let clip = Clip {
        x: x as _,
        y: y as _,
        width: w as _,
        height: h as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_rect(context, &framebuffer, x, y, w as u32, h as u32, color, clip)
}

pub async fn draw_line(context: &mut dyn WIPICContext, dst: WIPICIndirectPtr, x1: i32, y1: i32, x2: i32, y2: i32, pgc: WIPICWord) -> Result<()> {
    tracing::debug!("MC_grpDrawLine({:#x}, {x1}, {y1}, {x2}, {y2}, {pgc:#x})", dst.0);

    let framebuffer = FrameBuffer(read_generic(context, context.data_ptr(dst)?)?);
    let gctx: WIPICGraphicsContext = read_generic(context, pgc)?;
    let clip = Clip {
        x: 0,
        y: 0,
        width: framebuffer.0.width as _,
        height: framebuffer.0.height as _,
    };

    let color = framebuffer.pixel_to_color(gctx.fgpxl);
    primitives::draw_line(context, &framebuffer, x1, y1, x2, y2, color, clip)
}

pub async fn post_event(context: &mut dyn WIPICContext, id: i32, r#type: i32, param1: i32, param2: i32) -> Result<i32> {
    tracing::debug!("MC_grpPostEvent({id}, {type}, {param1}, {param2})");

    context.system().event_queue().push(Event::Notify { r#type, param1, param2 });

    Ok(0)
}

// it's not documented api, but lgt apps gets pointer via api call
pub async fn get_framebuffer_pointer(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<WIPICWord> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_POINTER({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.buf.0)
}

pub async fn get_framebuffer_width(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_WIDTH({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.width as _)
}

pub async fn get_framebuffer_height(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_HEIGHT({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.height as _)
}

pub async fn get_framebuffer_bpl(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPL({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.bpl as _)
}

pub async fn get_framebuffer_bpp(context: &mut dyn WIPICContext, framebuffer: WIPICIndirectPtr) -> Result<i32> {
    tracing::debug!("MC_GRP_GET_FRAME_BUFFER_BPP({:#x})", framebuffer.0);

    let framebuffer: WIPICFramebuffer = read_generic(context, context.data_ptr(framebuffer)?)?;

    Ok(framebuffer.bpp as _)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec};

    use test_utils::TestPlatform;

    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::ByteWrite;

    use crate::{MethodImpl, context::test::TestContext};

    use super::*;

    #[futures_test::test]
    async fn strings_are_positioned_at_the_requested_baseline() -> Result<()> {
        let system = System::new(Box::new(TestPlatform::new()), "test-pid", "test-aid", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        let destination = create_offscreen_framebuffer(&mut context, 32, 32).await?;
        let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(destination)?)?);
        framebuffer.write(&mut context, &[0; 32 * 32 * 2])?;
        let graphics = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
        init_context(&mut context, graphics).await?;
        set_context(&mut context, graphics, WIPICGraphicsContextIdx::FgPixelIdx, 0xffff).await?;
        let text = context.alloc_raw(3)?;
        context.write_bytes(text, &[0xb0, 0xa1, 0])?;
        draw_string(&mut context, destination, 2, 16, text, -1, graphics).await?;

        let bottom = 16 + get_font_descent(&mut context, 0).await?;
        let image = framebuffer.image(&mut context)?;
        assert!((0..16).any(|y| (0..32).any(|x| image.get_pixel(x, y).r != 0)));
        assert!((bottom..32).all(|y| (0..32).all(|x| image.get_pixel(x, y).r == 0)));
        Ok(())
    }

    #[futures_test::test]
    async fn encoded_framebuffer_region_can_be_loaded_as_an_image() -> Result<()> {
        let mut context = TestContext::new();
        let source = create_offscreen_framebuffer(&mut context, 3, 2).await?;
        let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(source)?)?);
        framebuffer.write(&mut context, &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0xf8, 0xe0, 0x07])?;
        let length_ptr = context.alloc_raw(4)?;
        let output = context.alloc_raw(4)?;
        for (x, y, width, height) in [(0, 0, 3, 2), (1, 1, 2, 1)] {
            let encoded = WIPICIndirectPtr(encode_image(&mut context, source, x, y, width, height, length_ptr).await?);
            assert_ne!(encoded.0, 0);
            let length: u32 = read_generic(&context, length_ptr)?;
            assert!(length > 0);
            assert_eq!(read_generic::<[u8; 2], _>(&context, context.data_ptr(encoded)?)?, *b"BM");
            create_image(&mut context, output, encoded, 0, length).await?;
            let handle: WIPICIndirectPtr = read_generic(&context, output)?;
            let image: WIPICImage = read_generic(&context, context.data_ptr(handle)?)?;
            let decoded = FrameBuffer(image.img).image(&mut context)?;
            assert_eq!((decoded.width(), decoded.height()), (width as u32, height as u32));
            let original = framebuffer.image(&mut context)?;
            for row in 0..height {
                for col in 0..width {
                    let expected = original.get_pixel(x + col, y + row);
                    let actual = decoded.get_pixel(col, row);
                    assert_eq!([actual.r, actual.g, actual.b], [expected.r, expected.g, expected.b]);
                }
            }
        }
        for (x, y, width, height) in [(-1, 0, 1, 1), (0, 0, 0, 1), (2, 1, 2, 1), (0, 0, i32::MAX, 1)] {
            assert_eq!(encode_image(&mut context, source, x, y, width, height, length_ptr).await?, 0);
            assert_eq!(read_generic::<u32, _>(&context, length_ptr)?, 0);
        }
        Ok(())
    }

    #[futures_test::test]
    async fn indexed_images_preserve_transparency_and_context_clipping() -> Result<()> {
        for (reserved, important, transparent) in [(1u16, 0x101u32, Some(1)), (1, 0x100, Some(0)), (0, 0x101, None), (1, 3, None)] {
            let mut bmp = vec![0u8; 70];
            bmp[..2].copy_from_slice(b"BM");
            bmp[2..6].copy_from_slice(&70u32.to_le_bytes());
            bmp[6..8].copy_from_slice(&reserved.to_le_bytes());
            bmp[10..14].copy_from_slice(&66u32.to_le_bytes());
            bmp[14..18].copy_from_slice(&40u32.to_le_bytes());
            bmp[18..22].copy_from_slice(&3u32.to_le_bytes());
            bmp[22..26].copy_from_slice(&1u32.to_le_bytes());
            bmp[26..28].copy_from_slice(&1u16.to_le_bytes());
            bmp[28..30].copy_from_slice(&8u16.to_le_bytes());
            bmp[34..38].copy_from_slice(&4u32.to_le_bytes());
            bmp[46..50].copy_from_slice(&3u32.to_le_bytes());
            bmp[50..54].copy_from_slice(&important.to_le_bytes());
            // Equal RGB values at distinct indices must retain independent transparency.
            bmp[54..66].copy_from_slice(&[255, 0, 255, 0, 255, 0, 255, 0, 0, 0, 0, 0]);
            bmp[66..70].copy_from_slice(&[0, 1, 2, 0]);

            let mut context = TestContext::new();
            let buffer = context.alloc(bmp.len() as u32)?;
            context.write_bytes(context.data_ptr(buffer)?, &bmp)?;
            let output = context.alloc_raw(4)?;
            create_image(&mut context, output, buffer, 0, bmp.len() as u32).await?;
            let image: WIPICIndirectPtr = read_generic(&context, output)?;
            let destination = create_offscreen_framebuffer(&mut context, 4, 1).await?;
            let framebuffer = FrameBuffer(read_generic(&context, context.data_ptr(destination)?)?);
            let graphics_context = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
            init_context(&mut context, graphics_context).await?;
            let clip = context.alloc_raw(16)?;
            write_generic(&mut context, clip, [2i32, 0, 3, 1])?;
            for clip_ptr in [clip, 0] {
                set_context(&mut context, graphics_context, WIPICGraphicsContextIdx::ClipIdx, clip_ptr).await?;
                framebuffer.write(&mut context, &[0xe0, 0x07, 0xe0, 0x07, 0xe0, 0x07, 0xe0, 0x07])?;
                draw_image(&mut context, destination, 1, 0, 3, 1, image, 0, 0, graphics_context).await?;

                let drawn = framebuffer.image(&mut context)?;
                assert_eq!(drawn.get_pixel(0, 0).g, 255);
                for index in 0..3 {
                    let color = drawn.get_pixel(index + 1, 0);
                    let expected = if transparent == Some(index) || (clip_ptr != 0 && index != 1) {
                        [0, 255, 0]
                    } else if index == 2 {
                        [0, 0, 0]
                    } else {
                        [255, 0, 255]
                    };
                    assert_eq!([color.r, color.g, color.b], expected);
                }
            }
        }
        Ok(())
    }

    #[futures_test::test]
    async fn context_values_are_written_to_the_output_pointer() -> Result<()> {
        let mut context = TestContext::new();
        let ptr_context = context.alloc_raw(size_of::<WIPICGraphicsContext>() as u32)?;
        let input = context.alloc_raw(16)?;
        let output = context.alloc_raw(20)?;
        init_context(&mut context, ptr_context).await?;

        let set = set_context.into_body();
        let get = get_context.into_body();
        for (op, value) in [
            (1, 0x12345678),
            (2, 0x87654321),
            (3, 0xff00ff),
            (4, 128),
            (5, 0x1001),
            (6, 42),
            (7, 8),
            (8, 1),
        ] {
            set.call(&mut context, Box::new([ptr_context, op, value])).await?;
            write_generic(&mut context, output, [0xccccccccu32; 5])?;
            get.call(&mut context, Box::new([ptr_context, op, output])).await?;
            assert_eq!(
                read_generic::<[u32; 5], _>(&context, output)?,
                [value, 0xcccccccc, 0xcccccccc, 0xcccccccc, 0xcccccccc]
            );
        }

        write_generic(&mut context, input, [-5i32, -8, 176, 220])?;
        set.call(&mut context, Box::new([ptr_context, 0, input])).await?;
        write_generic(&mut context, output, [999i32; 5])?;
        get.call(&mut context, Box::new([ptr_context, 0, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-5, -8, 176, 220, 999]);

        let graphics: WIPICGraphicsContext = read_generic(&context, ptr_context)?;
        assert_eq!(graphics.mask & 1, 1);
        set.call(&mut context, Box::new([ptr_context, 0, 0])).await?;
        let graphics: WIPICGraphicsContext = read_generic(&context, ptr_context)?;
        assert_eq!(graphics.mask & 1, 0);
        assert_eq!(graphics.fgpxl, 0x12345678);

        write_generic(&mut context, input, [-12i32, 34])?;
        set.call(&mut context, Box::new([ptr_context, 10, input])).await?;
        write_generic(&mut context, output, [999i32; 5])?;
        get.call(&mut context, Box::new([ptr_context, 10, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);

        get.call(&mut context, Box::new([ptr_context, 0xff, output])).await?;
        assert_eq!(read_generic::<[i32; 5], _>(&context, output)?, [-12, 34, 999, 999, 999]);
        Ok(())
    }
}
