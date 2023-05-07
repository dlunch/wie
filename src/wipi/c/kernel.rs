use super::{into_body, CContext, CError, CMethodBody, CResult, MethodImpl};

fn stub(_: &mut CContext) -> CResult<u32> {
    log::debug!("kernel stub called");

    Ok(0)
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
        into_body(reserved1),
    ]
}
