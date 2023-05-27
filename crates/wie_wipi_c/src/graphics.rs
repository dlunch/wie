use alloc::{vec, vec::Vec};

use core::mem::size_of;

use wie_base::util::write_generic;

use crate::{
    base::{CContext, CMemoryId, CMethodBody, CResult},
    method::MethodImpl,
};

#[repr(C)]
struct Framebuffer {
    width: u32,
    height: u32,
    bpl: u32,
    bpp: u32,
    id: CMemoryId,
}

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move {
        log::warn!("graphics stub{} called", id);

        Ok(0)
    };

    body.into_body()
}

async fn get_screen_frame_buffer(context: &mut dyn CContext, a0: u32) -> CResult<CMemoryId> {
    log::debug!("get_screen_frame_buffer({:#x})", a0);

    let franebuffer_data = context.alloc(320 * 480)?;

    let framebuffer = Framebuffer {
        // TODO: hardcoded
        width: 320,
        height: 480,
        bpl: 1280,
        bpp: 32,
        id: franebuffer_data,
    };

    let memory = context.alloc(size_of::<Framebuffer>() as u32)?;
    write_generic(context, context.data_ptr(memory)?, framebuffer)?;

    Ok(memory)
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
        gen_stub(25),
        gen_stub(26),
        gen_stub(27),
        gen_stub(28),
        gen_stub(29),
        gen_stub(30),
        gen_stub(31),
        gen_stub(32),
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
