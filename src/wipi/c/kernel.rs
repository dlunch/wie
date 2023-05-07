use super::{into_body, CContext, CError, CMethodBody, CResult, MethodImpl};

fn stub(_: &mut CContext) -> CResult<u32> {
    log::debug!("kernel stub called");

    Ok(0)
}

fn current_time(_: &mut CContext) -> CResult<u32> {
    log::debug!("current_time()");

    Ok(0)
}

fn def_timer(_: &mut CContext, a0: u32, a1: u32) -> CResult<()> {
    log::debug!("def_timer({:#x}, {:#x})", a0, a1);

    Ok(())
}

fn alloc(context: &mut CContext, size: u32) -> CResult<u32> {
    log::debug!("alloc({:#x})", size);

    context.alloc(size)
}

fn get_resource_id(_: &mut CContext, name: String, ptr_size: u32) -> CResult<u32> {
    log::debug!("get_resource_id({}, {:#x})", name, ptr_size);

    Ok(4321)
}

fn get_resource(_: &mut CContext, a0: u32, a1: u32, a2: u32) -> CResult<u32> {
    log::debug!("get_resource({:#x}, {:#x}, {:#x})", a0, a1, a2);

    Ok(1234)
}

pub fn get_kernel_method_table<M, F, R, P>(reserved1: M) -> Vec<CMethodBody>
where
    M: MethodImpl<F, R, CError, CContext, P>,
{
    vec![
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
        into_body(alloc),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(def_timer),
        into_body(stub),
        into_body(stub),
        into_body(current_time),
        into_body(stub),
        into_body(stub),
        into_body(get_resource_id),
        into_body(get_resource),
        into_body(reserved1),
    ]
}
