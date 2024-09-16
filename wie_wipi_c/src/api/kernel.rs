use alloc::{
    boxed::Box,
    format, str,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use wie_backend::Instant;
use wie_util::{read_generic, read_null_terminated_string, write_generic, write_null_terminated_string, Result, WieError};

use crate::{
    context::WIPICContext,
    method::{MethodBody, MethodImpl},
    WIPICMemoryId, WIPICMethodBody, WIPICWord,
};

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICTimer {
    unk1: WIPICWord,
    unk2: WIPICWord,
    unk3: WIPICWord,
    time: u64,

    param: WIPICWord,
    unk4: WIPICWord,
    fn_callback: WIPICWord,
}

#[derive(Pod, Zeroable, Copy, Clone)]
#[repr(C)]
struct ResourceHandle {
    name: [u8; 32], // TODO hardcoded max size
}

fn gen_stub(_id: WIPICWord, name: &'static str) -> WIPICMethodBody {
    let body = move |_: &mut dyn WIPICContext| async move { Err::<(), _>(WieError::Unimplemented(name.into())) };

    body.into_body()
}

async fn current_time(context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::debug!("MC_knlCurrentTime()");

    Ok(context.system().platform().now().raw() as WIPICWord)
}

async fn get_system_property(_context: &mut dyn WIPICContext, id: String, p_out: WIPICWord, buf_size: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_knlGetSystemProperty({}, {:#x}, {})", id, p_out, buf_size);

    Ok(0)
}

async fn def_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord, fn_callback: WIPICWord) -> Result<()> {
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

async fn set_timer(
    context: &mut dyn WIPICContext,
    ptr_timer: WIPICWord,
    timeout_low: WIPICWord,
    timeout_high: WIPICWord,
    param: WIPICWord,
) -> Result<()> {
    tracing::debug!("MC_knlSetTimer({:#x}, {:#x}, {:#x}, {:#x})", ptr_timer, timeout_low, timeout_high, param);

    struct TimerCallback {
        ptr_timer: u32,
        wakeup: Instant,
        param: WIPICWord,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for TimerCallback {
        #[tracing::instrument(name = "timer", skip_all)]
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICWord> {
            let timer: WIPICTimer = read_generic(context, self.ptr_timer)?;

            context.system().sleep(self.wakeup).await;

            context.call_function(timer.fn_callback, &[self.ptr_timer, self.param]).await?;

            Ok(0)
        }
    }

    let wakeup = context.system().platform().now() + (((timeout_high as u64) << 32) | (timeout_low as u64)) as _;

    context.spawn(Box::new(TimerCallback { ptr_timer, wakeup, param }))?;

    Ok(())
}

async fn unset_timer(_: &mut dyn WIPICContext, a0: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlUnsetTimer({:#x})", a0);

    Ok(())
}

async fn alloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlAlloc({:#x})", size);

    context.alloc(size)
}

async fn calloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlCalloc({:#x})", size);

    let memory = context.alloc(size)?;

    let zero = iter::repeat(0).take(size as usize).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

async fn free(context: &mut dyn WIPICContext, memory: WIPICMemoryId) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlFree({:#x})", memory.0);

    context.free(memory)?;

    Ok(memory)
}

async fn get_resource_id(context: &mut dyn WIPICContext, name: String, ptr_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResourceID({}, {:#x})", name, ptr_size);

    let size = context.get_resource_size(&name).await?; // TODO error handling

    // TODO it leaks handle every time.. should we assign id for every file?
    let name_bytes = name.as_bytes();
    let mut handle = ResourceHandle { name: [0; 32] };
    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    let ptr_handle = context.alloc_raw(size_of::<ResourceHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;
    write_generic(context, ptr_size, size)?;

    Ok(ptr_handle as _)
}

async fn get_resource(context: &mut dyn WIPICContext, id: WIPICWord, buf: WIPICMemoryId, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResource({}, {:#x}, {})", id, buf.0, buf_size);

    let handle: ResourceHandle = read_generic(context, id)?;
    let name_length = handle.name.iter().position(|&c| c == 0).unwrap_or(handle.name.len());
    let name = str::from_utf8(&handle.name[..name_length]).unwrap();

    let data = context.read_resource(name).await?;

    if data.len() as u32 > buf_size {
        return Ok(-1);
    }

    context.write_bytes(context.data_ptr(buf)?, &data)?;

    Ok(0)
}

async fn printk(context: &mut dyn WIPICContext, format: String, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("stub MC_knlPrintk({}, {:#x}, {:#x}, {:#x}, {:#x})", format, a0, a1, a2, a3);

    let result = sprintf(context, &format, &[a0, a1, a2, a3])?;

    tracing::info!("printk: {}", result);

    Ok(())
}

async fn sprintk(context: &mut dyn WIPICContext, dest: WIPICWord, format: String, a0: u32, a1: u32, a2: u32, a3: u32) -> Result<WIPICWord> {
    tracing::debug!("MC_knlSprintk({:#x}, {}, {:#x}, {:#x}, {:#x}, {:#x})", dest, format, a0, a1, a2, a3);

    let result = sprintf(context, &format, &[a0, a1, a2, a3])?;

    write_null_terminated_string(context, dest, &result)?;

    Ok(result.len() as _)
}

async fn get_total_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetTotalMemory()");

    Ok(0x100000) // TODO hardcoded
}

async fn get_free_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetFreeMemory()");

    Ok(0x100000) // TODO hardcoded
}

fn sprintf(context: &mut dyn WIPICContext, format: &str, args: &[u32]) -> Result<String> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars();
    let mut arg_iter = args.iter();

    while let Some(x) = chars.next() {
        if x == '%' {
            let format = chars.next().unwrap();
            match format {
                '%' => result.push('%'),
                'd' => result += &arg_iter.next().unwrap().to_string(),
                's' => {
                    let ptr = arg_iter.next().unwrap();
                    let str = read_null_terminated_string(context, *ptr)?;

                    result += &str;
                }
                'c' => {
                    let byte = arg_iter.next().unwrap();
                    result.push(*byte as u8 as char)
                }
                'x' => {
                    let byte = arg_iter.next().unwrap();
                    result += &format!("{:x}", byte)
                }
                _ => unimplemented!("Unknown format: {}", format),
            }
        } else {
            result.push(x);
        }
    }

    Ok(result)
}

async fn get_cur_program_id(_context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_knlGetCurProgramID()");

    Ok(1)
}

pub fn get_kernel_method_table<M, F, R, P>(reserved1: M) -> Vec<WIPICMethodBody>
where
    M: MethodImpl<F, R, WieError, P>,
{
    vec![
        printk.into_body(),
        sprintk.into_body(),
        gen_stub(2, "MC_knlGetExecNames"),
        gen_stub(3, "MC_knlExecute"),
        gen_stub(4, "MC_knlMExecute"),
        gen_stub(5, "MC_knlLoad"),
        gen_stub(6, "MC_knlMLoad"),
        gen_stub(7, "MC_knlExit"),
        gen_stub(8, "MC_knlProgramStop"),
        get_cur_program_id.into_body(),
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
        get_total_memory.into_body(),
        get_free_memory.into_body(),
        def_timer.into_body(),
        set_timer.into_body(),
        unset_timer.into_body(),
        current_time.into_body(),
        get_system_property.into_body(),
        gen_stub(30, "MC_knlSetSystemProperty"),
        get_resource_id.into_body(),
        get_resource.into_body(),
        reserved1.into_body(),
        gen_stub(34, "MC_knlReserved2"),
        gen_stub(35, "MC_knlReserved3"),
        gen_stub(36, "MC_knlReserved4"),
        gen_stub(37, "MC_knlReserved5"),
        gen_stub(38, "MC_knlReserved6"),
        gen_stub(39, "MC_knlReserved7"),
        gen_stub(40, "MC_knlReserved8"),
        gen_stub(41, "MC_knlReserved9"),
        gen_stub(42, "MC_knlReserved10"),
        gen_stub(43, "MC_knlReserved11"),
        gen_stub(44, "OEMC_knlSendMessage"),
        gen_stub(45, "OEMC_knlSetTimerEx"),
        gen_stub(46, "OEMC_knlGetSystemState"),
        gen_stub(47, "OEMC_knlCreateSystemProgressBar"),
        gen_stub(48, "OEMC_knlSetSystemProgressBar"),
        gen_stub(49, "OEMC_knlDestroySystemProgressBar"),
        gen_stub(50, "OEMC_knlExecuteEx"),
        gen_stub(51, "OEMC_knlGetProcAddress"),
        gen_stub(52, "OEMC_knlUnload"),
        gen_stub(53, "OEMC_knlCreateSysMessageBox"),
        gen_stub(54, "OEMC_knlDestroySysMessageBox"),
        gen_stub(55, "OEMC_knlGetProgramIDList"),
        gen_stub(56, "OEMC_knlGetProgramInfo"),
        gen_stub(57, "MC_knlReserved12"),
        gen_stub(58, "MC_knlReserved13"),
        gen_stub(59, "OEMC_knlCreateAppPrivateArea"),
        gen_stub(60, "OEMC_knlGetAppPrivateArea"),
        gen_stub(61, "OEMC_knlCreateLibPrivateArea"),
        gen_stub(62, "OEMC_knlGetLibPrivateArea"),
        gen_stub(63, "OEMC_knlGetPlatformVersion"),
        gen_stub(64, "OEMC_knlGetToken"),
    ]
}
