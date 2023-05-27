use alloc::{string::String, vec, vec::Vec};
use core::iter;

use wie_base::util::write_generic;

use crate::{
    base::{CContext, CError, CMemoryId, CMethodBody, CResult},
    method::MethodImpl,
};

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move {
        log::warn!("stub kernel{}", id);

        Ok(0)
    };

    body.into_body()
}

async fn current_time(context: &mut dyn CContext) -> CResult<u32> {
    log::debug!("current_time()");

    Ok(context.backend().time().now().raw() as u32)
}

async fn def_timer(_: &mut dyn CContext, a0: u32, a1: u32) -> CResult<()> {
    log::warn!("stub def_timer({:#x}, {:#x})", a0, a1);

    Ok(())
}

async fn alloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    log::debug!("alloc({:#x})", size);

    context.alloc(size)
}

async fn calloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    log::debug!("calloc({:#x})", size);

    let memory = context.alloc(size)?;

    let zero = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

async fn free(context: &mut dyn CContext, memory: CMemoryId) -> CResult<CMemoryId> {
    log::debug!("free({:#x})", memory.0);

    context.free(memory)?;

    Ok(memory)
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

async fn get_resource(context: &mut dyn CContext, id: u32, buf: CMemoryId, buf_size: u32) -> CResult<i32> {
    log::debug!("get_resource({}, {:#x}, {})", id, buf.0, buf_size);

    let size = context.backend().resource().size(id);

    if size > buf_size {
        return Ok(-1);
    }

    let data = context.backend().resource().data(id).to_vec(); // TODO: can we avoid to_vec()?

    context.write_bytes(context.data_ptr(buf)?, &data)?;

    Ok(0)
}

pub fn get_kernel_method_table<M, F, R, P>(reserved1: M) -> Vec<CMethodBody>
where
    M: MethodImpl<F, R, CError, P>,
{
    vec![
        gen_stub(0),
        gen_stub(1),
        gen_stub(2),
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
        alloc.into_body(),
        calloc.into_body(),
        free.into_body(),
        gen_stub(23),
        gen_stub(24),
        def_timer.into_body(),
        gen_stub(26),
        gen_stub(27),
        current_time.into_body(),
        gen_stub(29),
        gen_stub(30),
        get_resource_id.into_body(),
        get_resource.into_body(),
        reserved1.into_body(),
    ]
}
