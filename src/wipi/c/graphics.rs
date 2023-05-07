use super::{into_body, CContext, CMethodBody, CResult};

fn stub(_: &mut CContext) -> CResult<u32> {
    log::debug!("graphics stub called");

    Ok(0)
}

fn get_screen_frame_buffer(_: &mut CContext, a0: u32) -> CResult<u32> {
    log::debug!("get_screen_frame_buffer({:#x})", a0);

    Ok(1234)
}

pub fn get_graphics_method_table() -> Vec<CMethodBody> {
    vec![into_body(stub), into_body(stub), into_body(get_screen_frame_buffer)]
}
