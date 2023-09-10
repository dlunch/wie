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

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented graphics{}", id)) };

    body.into_body()
}

async fn get_screen_framebuffer(context: &mut dyn CContext, a0: u32) -> CResult<CMemoryId> {
    log::debug!("MC_grpGetScreenFrameBuffer({:#x})", a0);

    let framebuffer = WIPICFramebuffer::from_screen_canvas(context)?;

    let memory = context.alloc(size_of::<WIPICFramebuffer>() as u32)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
}

async fn create_image(context: &mut dyn CContext, ptr_image: u32, image_data: CMemoryId, offset: u32, len: u32) -> CResult<u32> {
    log::debug!("MC_grpCreateImage({:#x}, {:#x}, {:#x}, {:#x})", ptr_image, image_data.0, offset, len);

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
    log::debug!(
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
    log::debug!(
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
        gen_stub(0),
        gen_stub(1),
        get_screen_framebuffer.into_body(),
        gen_stub(3),
        gen_stub(4),
        gen_stub(5),
        gen_stub(6),
        gen_stub(7),
        gen_stub(8),
        gen_stub(9),
        gen_stub(10),
        gen_stub(11),
        gen_stub(12),
        draw_image.into_body(),
        gen_stub(14),
        gen_stub(15),
        gen_stub(16),
        gen_stub(17),
        gen_stub(18),
        gen_stub(19),
        gen_stub(20),
        flush.into_body(),
        gen_stub(22),
        gen_stub(23),
        gen_stub(24),
        gen_stub(25),
        gen_stub(26),
        gen_stub(27),
        gen_stub(28),
        gen_stub(29),
        gen_stub(30),
        gen_stub(31),
        create_image.into_body(),
        gen_stub(33),
        gen_stub(34),
        gen_stub(35),
        gen_stub(36),
        gen_stub(37),
        gen_stub(38),
        gen_stub(39),
        gen_stub(40),
        gen_stub(41),
        gen_stub(42),
        gen_stub(43),
        gen_stub(44),
        gen_stub(45),
        gen_stub(46),
        gen_stub(47),
        gen_stub(48),
        gen_stub(49),
        gen_stub(50),
        gen_stub(51),
        gen_stub(52),
        gen_stub(53),
        gen_stub(54),
        gen_stub(55),
        gen_stub(56),
        gen_stub(57),
        gen_stub(58),
        gen_stub(59),
        gen_stub(60),
        gen_stub(61),
        gen_stub(62),
        gen_stub(63),
        gen_stub(64),
        gen_stub(65),
        gen_stub(66),
        gen_stub(67),
        gen_stub(68),
        gen_stub(69),
        gen_stub(70),
        gen_stub(71),
        gen_stub(72),
        gen_stub(73),
        gen_stub(74),
        gen_stub(75),
        gen_stub(76),
        gen_stub(77),
        gen_stub(78),
        gen_stub(79),
    ]
}
