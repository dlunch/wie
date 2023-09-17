use alloc::{boxed::Box, format, string::String, vec, vec::Vec};
use core::{cell::Ref, iter};

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

fn gen_stub(id: u32, name: &'static str) -> CMethodBody {
    let body = move |_: &mut dyn CContext| async move { Err::<(), _>(anyhow::anyhow!("Unimplemented kernel{}: {}", id, name)) };

    body.into_body()
}

async fn current_time(context: &mut dyn CContext) -> CResult<u32> {
    tracing::debug!("MC_knlCurrentTime()");

    Ok(context.backend().time().now().raw() as u32)
}

async fn get_system_property(context: &mut dyn CContext, p_id: u32, p_out: u32, buf_size: u32) -> CResult<i32> {
    let value = format!("prop{}\x00", p_id);
    tracing::warn!(
        "stub MC_knlGetSystemProperty({:#x}, {:#x}, {}) - returning {}",
        p_id,
        p_out,
        buf_size,
        &value,
    );

    if (buf_size as i64) < (value.len() as i64) {
        return Ok(-18); // M_E_SHORTBUF
    }
    context.write_bytes(p_out, value.as_bytes())?;

    Ok(0)
}

async fn def_timer(context: &mut dyn CContext, ptr_timer: u32, fn_callback: u32) -> CResult<()> {
    tracing::debug!("MC_knlDefTimer({:#x}, {:#x})", ptr_timer, fn_callback);

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
    tracing::debug!("MC_knlSetTimer({:#x}, {:#x}, {:#x}, {:#x})", ptr_timer, timeout_high, timeout_low, param);

    let timer: WIPICTimer = read_generic(context, ptr_timer)?;

    struct TimerCallback {
        timer: WIPICTimer,
        timeout: u64,
        param: u32,
    }

    #[async_trait::async_trait(?Send)]
    impl MethodBody<CError> for TimerCallback {
        #[tracing::instrument(name = "timer", skip_all)]
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
    tracing::warn!("stub MC_knlUnsetTimer({:#x})", a0);

    todo!();
}

async fn alloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    tracing::debug!("MC_knlAlloc({:#x})", size);

    context.alloc(size)
}

async fn calloc(context: &mut dyn CContext, size: u32) -> CResult<CMemoryId> {
    tracing::debug!("MC_knlCalloc({:#x})", size);

    let memory = context.alloc(size)?;

    let zero = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

async fn free(context: &mut dyn CContext, memory: CMemoryId) -> CResult<CMemoryId> {
    tracing::debug!("MC_knlFree({:#x})", memory.0);

    context.free(memory)?;

    Ok(memory)
}

async fn get_resource_id(context: &mut dyn CContext, name: String, ptr_size: u32) -> CResult<i32> {
    tracing::debug!("MC_knlGetResourceID({}, {:#x})", name, ptr_size);

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
    tracing::debug!("MC_knlGetResource({}, {:#x}, {})", id, buf.0, buf_size);

    let size = context.backend().resource().size(id);

    if size > buf_size {
        return Ok(-1);
    }

    let backend1 = context.backend().clone();
    let data = Ref::map(backend1.resource(), |x| x.data(id));

    context.write_bytes(context.data_ptr(buf)?, &data)?;

    Ok(0)
}

pub fn get_kernel_method_table<M, F, R, P>(reserved1: M) -> Vec<CMethodBody>
where
    M: MethodImpl<F, R, CError, P>,
{
    vec![
        gen_stub(0, "MC_knlPrintk"),
        gen_stub(1, "MC_knlSprintk"),
        gen_stub(2, "MC_knlGetExecNames"),
        gen_stub(3, "MC_knlExecute"),
        gen_stub(4, "MC_knlMExecute"),
        gen_stub(5, "MC_knlLoad"),
        gen_stub(6, "MC_knlMLoad"),
        gen_stub(7, "MC_knlExit"),
        gen_stub(8, "MC_knlProgramStop"),
        gen_stub(9, "MC_knlGetCurProgramID"),
        gen_stub(10, "MC_knlGetParentProgramID"),
        gen_stub(11, "MC_knlGetAppManagerID"),
        gen_stub(12, "MC_knlGetProgramInfo"),
        gen_stub(13, "MC_knlGetAccessLevel"),
        gen_stub(14, "MC_knlGetProgramName"),
        gen_stub(15, "MC_knlCreateSharedBuf"),
        gen_stub(16, "MC_knlDestroySharedBuf"),
        gen_stub(17, "MC_knlGetSharedBuf"),
        gen_stub(18, "MC_knlGetSharedBufSize"),
        gen_stub(19, "MC_knlResizeSharedBuf"),
        alloc.into_body(),
        calloc.into_body(),
        free.into_body(),
        gen_stub(23, "MC_knlGetTotalMemory"),
        gen_stub(24, "MC_knlGetFreeMemory"),
        def_timer.into_body(),
        set_timer.into_body(),
        unset_timer.into_body(),
        current_time.into_body(),
        get_system_property.into_body(),
        gen_stub(30, "MC_knlSetSystemProperty"),
        get_resource_id.into_body(),
        get_resource.into_body(),
        reserved1.into_body(),
        // gen_stub(34, "MC_knlReserved2"),
        // gen_stub(35, "MC_knlReserved3"),
        // gen_stub(36, "MC_knlReserved4"),
        // gen_stub(37, "MC_knlReserved5"),
        // gen_stub(38, "MC_knlReserved6"),
        // gen_stub(39, "MC_knlReserved7"),
        // gen_stub(40, "MC_knlReserved8"),
        // gen_stub(41, "MC_knlReserved9"),
        // gen_stub(42, "MC_knlReserved10"),
        // gen_stub(43, "MC_knlReserved11"),
        // gen_stub(44, "OEMC_knlSendMessage"),
        // gen_stub(45, "OEMC_knlSetTimerEx"),
        // gen_stub(46, "OEMC_knlGetSystemState"),
        // gen_stub(47, "OEMC_knlCreateSystemProgressBar"),
        // gen_stub(48, "OEMC_knlSetSystemProgressBar"),
        // gen_stub(49, "OEMC_knlDestroySystemProgressBar"),
        // gen_stub(50, "OEMC_knlExecuteEx"),
        // gen_stub(51, "OEMC_knlGetProcAddress"),
        // gen_stub(52, "OEMC_knlUnload"),
        // gen_stub(53, "OEMC_knlCreateSysMessageBox"),
        // gen_stub(54, "OEMC_knlDestroySysMessageBox"),
        // gen_stub(55, "OEMC_knlGetProgramIDList"),
        // gen_stub(56, "OEMC_knlGetProgramInfo"),
        // gen_stub(57, "MC_knlReserved12"),
        // gen_stub(58, "MC_knlReserved13"),
        // gen_stub(59, "OEMC_knlCreateAppPrivateArea"),
        // gen_stub(60, "OEMC_knlGetAppPrivateArea"),
        // gen_stub(61, "OEMC_knlCreateLibPrivateArea"),
        // gen_stub(62, "OEMC_knlGetLibPrivateArea"),
        // gen_stub(63, "OEMC_knlGetPlatformVersion"),
        // gen_stub(64, "OEMC_knlGetToken"),
    ]
}
