use std::mem::size_of;

use super::{into_body, CContext, CMethodBody, CResult};

#[repr(C)]
struct Framebuffer {
    width: u32,
    height: u32,
    bpl: u32,
    bpp: u32,
    id: u32,
}

fn stub(_: &mut CContext) -> CResult<u32> {
    log::debug!("graphics stub called");

    Ok(0)
}

fn get_screen_frame_buffer(context: &mut CContext, a0: u32) -> CResult<u32> {
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

    let data = unsafe { std::slice::from_raw_parts(&framebuffer as *const _ as *const u8, std::mem::size_of::<Framebuffer>()) }; // TODO: we should have converter
    context.write_raw(ptr_framebuffer, data)?;

    let address = context.alloc(4)?;
    let data = unsafe { std::slice::from_raw_parts(&ptr_framebuffer as *const _ as *const u8, std::mem::size_of::<u32>()) };
    context.write_raw(address, data)?;

    Ok(address)
}

fn get_display_info(_: &mut CContext, a0: u32, a1: u32) -> CResult<u32> {
    log::debug!("get_display_info({:#x}, {:#x})", a0, a1);

    Ok(0)
}

pub fn get_graphics_method_table() -> Vec<CMethodBody> {
    vec![
        into_body(stub),
        into_body(stub),
        into_body(get_screen_frame_buffer),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(get_display_info),
    ]
}
