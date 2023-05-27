use alloc::{vec, vec::Vec};

use core::mem::size_of;

use wie_base::util::write_generic;

use crate::{
    base::{CContext, CMethodBody, CResult},
    method::MethodImpl,
};

#[repr(C)]
struct Framebuffer {
    width: u32,
    height: u32,
    bpl: u32,
    bpp: u32,
    id: u32,
}

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move {
        log::warn!("graphics stub{} called", id);

        Ok(0)
    };

    body.into_body()
}

async fn get_screen_frame_buffer(context: &mut dyn CContext, a0: u32) -> CResult<u32> {
    log::debug!("get_screen_frame_buffer({:#x})", a0);

    let ptr_framebuffer = context.alloc(size_of::<Framebuffer>() as u32)?;
    let framebuffer = Framebuffer {
        // TODO: hardcoded
        width: 320,
        height: 480,
        bpl: 1280,
        bpp: 32,
        id: 0,
    };

    write_generic(context, ptr_framebuffer, framebuffer)?;

    let address = context.alloc(4)?;
    write_generic(context, address, ptr_framebuffer)?;

    Ok(address)
}

async fn get_display_info(_: &mut dyn CContext, a0: u32, a1: u32) -> CResult<u32> {
    log::debug!("get_display_info({:#x}, {:#x})", a0, a1);

    Ok(0)
}

pub fn get_graphics_method_table() -> Vec<CMethodBody> {
    vec![
        gen_stub(0),
        gen_stub(1),
        get_screen_frame_buffer.into_body(),
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
        gen_stub(13),
        gen_stub(14),
        gen_stub(15),
        gen_stub(16),
        gen_stub(17),
        gen_stub(18),
        gen_stub(19),
        gen_stub(20),
        gen_stub(21),
        gen_stub(22),
        gen_stub(23),
        get_display_info.into_body(),
    ]
}
