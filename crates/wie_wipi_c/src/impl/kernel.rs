use alloc::{boxed::Box, string::String, vec, vec::Vec};
use core::iter;

use bytemuck::{Pod, Zeroable};

use wie_base::util::{read_generic, write_generic};

use crate::{
    base::{CContext, CError, CMemoryId, CMethodBody, CResult},
    method::{MethodBody, MethodImpl},
};

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICTimer {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    time: u64,

    param: u32,
    unk4: u32,
    fn_callback: u32,
}

fn gen_stub(id: u32) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented kernel{}", id)) };

    body.into_body()
}

async fn current_time(context: &mut dyn CContext) -> CResult<u32> {
    log::debug!("MC_knlCurrentTime()");

    Ok(context.backend().time().now().raw() as u32)
}

async fn def_timer(context: &mut dyn CContext, ptr_timer: u32, fn_callback: u32) -> CResult<()> {
    log::debug!("MC_knlDefTimer({:#x}, {:#x})", ptr_timer, fn_callback);

    let timer = WIPICTimer {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        time: 0,
        param: 0,
        unk4: 0,
        fn_callback,
    };

    write_generic(context, ptr_timer, timer)?;

    Ok(())
}

async fn set_timer(context: &mut dyn CContext, ptr_timer: u32, timeout_low: u32, timeout_high: u32, param: u32) -> CResult<()> {
    log::debug!("MC_knlSetTimer({:#x}, {:#x}, {:#x}, {:#x})", ptr_timer, timeout_high, timeout_low, param);

    let timer: WIPICTimer = read_generic(context, ptr_timer)?;

    struct TimerCallback {
        timer: WIPICTimer,
        timeout: u64,
        param: u32,
    }

    #[async_trait::async_trait(?Send)]
    impl MethodBody<CError> for TimerCallback {
        async fn call(&self, context: &mut dyn CContext, _: &[u32]) -> Result<u32, CError> {
            context.sleep(self.timeout).await;

            context.call_method(self.timer.fn_callback, &[self.param]).await?;

            Ok(0)
        }
    }

    context.spawn(Box::new(TimerCallback {
        timer,
        timeout: ((timeout_high as u64) << 32) | (timeout_low as u64),
        param,
    }))?;

    Ok(())
}

async fn unset_timer(_: &mut dyn CContext, a0: u32) -> CResult<()> {
    log::warn!("stub MC_knlUnsetTimer({:#x})", a0);

    todo!();
}

async fn alloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    log::debug!("MC_knlAlloc({:#x})", size);

    context.alloc(size)
}

async fn calloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    log::debug!("MC_knlCalloc({:#x})", size);

    let memory = context.alloc(size)?;

    let zero = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

async fn free(context: &mut dyn CContext, memory: CMemoryId) -> CResult<CMemoryId> {
    log::debug!("MC_knlFree({:#x})", memory.0);

    context.free(memory)?;

    Ok(memory)
}

async fn get_resource_id(context: &mut dyn CContext, name: String, ptr_size: u32) -> CResult<i32> {
    log::debug!("MC_knlGetResourceID({}, {:#x})", name, ptr_size);

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
    log::debug!("MC_knlGetResource({}, {:#x}, {})", id, buf.0, buf_size);

    let size = context.backend().resource().size(id);

    if size > buf_size {
        return Ok(-1);
    }

    let data = context.backend().clone().resource().data(id).to_vec(); // TODO: can we avoid to_vec()?

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
        set_timer.into_body(),
        unset_timer.into_body(),
        current_time.into_body(),
        gen_stub(29),
        gen_stub(30),
        get_resource_id.into_body(),
        get_resource.into_body(),
        reserved1.into_body(),
    ]
}
