use crate::util::write_generic;

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

fn get_resource_id(context: &mut CContext, name: String, ptr_size: u32) -> CResult<i32> {
    log::debug!("get_resource_id({}, {:#x})", name, ptr_size);

    let id = context.backend().resource().id(&name);
    if id.is_none() {
        return Ok(-1);
    }
    let id = id.unwrap();
    let size = context.backend().resource().size(id);

    write_generic(context, ptr_size, size)?;

    Ok(id as _)
}

fn get_resource(context: &mut CContext, id: u32, buf: u32, buf_size: u32) -> CResult<i32> {
    log::debug!("get_resource({}, {:#x}, {})", id, buf, buf_size);

    let size = context.backend().resource().size(id);

    if size > buf_size {
        return Ok(-1);
    }

    let data = context.backend().resource().data(id).to_vec(); // TODO: can we avoid to_vec()?
    context.write_bytes(buf, &data)?;

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
