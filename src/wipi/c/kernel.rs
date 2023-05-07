use super::{into_body, CContext, CError, CMethodBody, CResult, MethodImpl};

fn stub(_: &mut CContext) -> CResult<u32> {
    log::debug!("kernel stub called");

    Ok(0)
}

fn current_time(_: &mut CContext) -> CResult<u32> {
    log::debug!("current_time()");

    Ok(0)
}

fn alloc(context: &mut CContext, size: u32) -> CResult<u32> {
    log::debug!("alloc({:#x})", size);

    context.alloc(size)
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
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(current_time),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(stub),
        into_body(reserved1),
    ]
}
