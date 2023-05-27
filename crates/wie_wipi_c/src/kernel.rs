use alloc::{string::String, vec, vec::Vec};

use wie_base::util::{read_generic, write_generic};

use crate::{
    base::{CContext, CError, CMethodBody, CResult},
    method::MethodImpl,
};

async fn stub(_: &mut dyn CContext) -> CResult<u32> {
    log::debug!("kernel stub called");

    Ok(0)
}

async fn current_time(_: &mut dyn CContext) -> CResult<u32> {
    log::debug!("current_time()");

    Ok(0)
}

async fn def_timer(_: &mut dyn CContext, a0: u32, a1: u32) -> CResult<()> {
    log::debug!("def_timer({:#x}, {:#x})", a0, a1);

    Ok(())
}

async fn alloc(context: &mut dyn CContext, size: u32) -> CResult<u32> {
    log::debug!("alloc({:#x})", size);

    let ptr = context.alloc(4)?;
    let data = context.alloc(size)?;

    write_generic(context, ptr, data)?;

    Ok(ptr)
}

async fn get_resource_id(context: &mut dyn CContext, name: String, ptr_size: u32) -> CResult<i32> {
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

async fn get_resource(context: &mut dyn CContext, id: u32, buf: u32, buf_size: u32) -> CResult<i32> {
    log::debug!("get_resource({}, {:#x}, {})", id, buf, buf_size);

    let size = context.backend().resource().size(id);

    if size > buf_size {
        return Ok(-1);
    }

    let data = context.backend().resource().data(id).to_vec(); // TODO: can we avoid to_vec()?

    let ptr: u32 = read_generic(context, buf)?;
    context.write_bytes(ptr, &data)?;

    Ok(0)
}

pub fn get_kernel_method_table<M, F, R, P>(reserved1: M) -> Vec<CMethodBody>
where
    M: MethodImpl<F, R, CError, P>,
{
    vec![
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        alloc.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        stub.into_body(),
        def_timer.into_body(),
        stub.into_body(),
        stub.into_body(),
        current_time.into_body(),
        stub.into_body(),
        stub.into_body(),
        get_resource_id.into_body(),
        get_resource.into_body(),
        reserved1.into_body(),
    ]
}
