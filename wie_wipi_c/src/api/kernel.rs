use alloc::{boxed::Box, format, str, string::String, vec::Vec};
use core::{iter, mem::size_of};

use bytemuck::{Pod, Zeroable};

use wie_backend::Instant;
use wie_util::{read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes, Result, WieError};

use crate::{context::WIPICContext, method::MethodBody, WIPICMemoryId, WIPICResult, WIPICWord};

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

pub async fn current_time(context: &mut dyn WIPICContext) -> Result<u64> {
    tracing::debug!("MC_knlCurrentTime()");

    Ok(context.system().platform().now().raw())
}

pub async fn get_system_property(_context: &mut dyn WIPICContext, ptr_id: WIPICWord, p_out: WIPICWord, buf_size: WIPICWord) -> Result<i32> {
    tracing::warn!("stub MC_knlGetSystemProperty({:#x}, {:#x}, {:#x})", ptr_id, p_out, buf_size);

    Ok(-9) // M_E_INVALID
}

pub async fn set_system_property(_context: &mut dyn WIPICContext, ptr_id: WIPICWord, ptr_value: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlSetSystemProperty({:#x}, {:#x})", ptr_id, ptr_value);

    Ok(())
}

pub async fn def_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord, fn_callback: WIPICWord) -> Result<()> {
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

pub async fn set_timer(
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
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            let timer: WIPICTimer = read_generic(context, self.ptr_timer)?;

            context.system().sleep(self.wakeup).await;

            context.call_function(timer.fn_callback, &[self.ptr_timer, self.param]).await?;

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    let wakeup = context.system().platform().now() + (((timeout_high as u64) << 32) | (timeout_low as u64)) as _;

    context.spawn(Box::new(TimerCallback { ptr_timer, wakeup, param }))?;

    Ok(())
}

pub async fn unset_timer(_: &mut dyn WIPICContext, a0: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlUnsetTimer({:#x})", a0);

    Ok(())
}

pub async fn alloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlAlloc({:#x})", size);

    context.alloc(size)
}

pub async fn calloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlCalloc({:#x})", size);

    let memory = context.alloc(size)?;

    let zero = iter::repeat_n(0, size as _).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

pub async fn free(context: &mut dyn WIPICContext, memory: WIPICMemoryId) -> Result<WIPICMemoryId> {
    tracing::debug!("MC_knlFree({:#x})", memory.0);

    context.free(memory)?;

    Ok(memory)
}

pub async fn get_resource_id(context: &mut dyn WIPICContext, ptr_name: WIPICWord, ptr_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResourceID({:#x}, {:#x})", ptr_name, ptr_size);

    let name = String::from_utf8(read_null_terminated_string_bytes(context, ptr_name)?).unwrap();

    let size = context.get_resource_size(&name).await?;

    if size.is_none() {
        return Ok(-12); // M_E_NOENT
    }

    let size = size.unwrap();

    // TODO it leaks handle every time.. should we assign id for every file?
    let name_bytes = name.as_bytes();
    let mut handle = ResourceHandle { name: [0; 32] };
    handle.name[..name_bytes.len()].copy_from_slice(name_bytes);

    let ptr_handle = context.alloc_raw(size_of::<ResourceHandle>() as _)?;
    write_generic(context, ptr_handle, handle)?;
    write_generic(context, ptr_size, size as u32)?;

    Ok(ptr_handle as _)
}

pub async fn get_resource(context: &mut dyn WIPICContext, id: WIPICWord, buf: WIPICMemoryId, buf_size: WIPICWord) -> Result<i32> {
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

pub async fn printk(context: &mut dyn WIPICContext, ptr_format: WIPICWord, a0: WIPICWord, a1: WIPICWord, a2: WIPICWord, a3: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlPrintk({:#x}, {:#x}, {:#x}, {:#x}, {:#x})", ptr_format, a0, a1, a2, a3);

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let format_string = encoding_rs::EUC_KR.decode(&format_string).0;

    let result = sprintf(context, &format_string, &[a0, a1, a2, a3])?;

    tracing::info!("printk: {}", result);

    Ok(())
}

pub async fn sprintk(
    context: &mut dyn WIPICContext,
    dest: WIPICWord,
    ptr_format: WIPICWord,
    a0: WIPICWord,
    a1: WIPICWord,
    a2: WIPICWord,
    a3: WIPICWord,
) -> Result<WIPICWord> {
    tracing::debug!(
        "MC_knlSprintk({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        dest,
        ptr_format,
        a0,
        a1,
        a2,
        a3
    );

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let format_string = encoding_rs::EUC_KR.decode(&format_string).0;

    let result = sprintf(context, &format_string, &[a0, a1, a2, a3])?;

    let result_bytes = encoding_rs::EUC_KR.encode(&result).0;

    write_null_terminated_string_bytes(context, dest, &result_bytes)?;

    Ok(result.len() as _)
}

pub async fn get_total_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetTotalMemory()");

    Ok(0x100000) // TODO hardcoded
}

pub async fn get_free_memory(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetFreeMemory()");

    Ok(0x100000) // TODO hardcoded
}

fn sprintf(context: &mut dyn WIPICContext, format: &str, args: &[u32]) -> Result<String> {
    let mut result = String::with_capacity(format.len());
    let mut chars = format.chars();
    let mut arg_iter = args.iter();

    while let Some(x) = chars.next() {
        if x == '%' {
            let mut flag = None;
            let mut width = None;
            loop {
                let c = chars.next().unwrap();
                match c {
                    '%' => {
                        result.push('%');
                        break;
                    }
                    'd' => {
                        let arg = *arg_iter.next().unwrap();
                        if let Some('0') = flag {
                            result += &format!("{:0width$}", arg, width = width.unwrap_or(0));
                        } else {
                            result += &format!("{:width$}", arg, width = width.unwrap_or(0));
                        }
                        break;
                    }
                    's' => {
                        let ptr = *arg_iter.next().unwrap();
                        if ptr == 0 {
                            result += "(null)";
                            break;
                        }

                        let str = read_null_terminated_string_bytes(context, ptr)?;
                        let str = encoding_rs::EUC_KR.decode(&str).0;

                        result += &str;
                        break;
                    }
                    'c' => {
                        let byte = arg_iter.next().unwrap();
                        result.push(*byte as u8 as char);
                        break;
                    }
                    'x' => {
                        let byte = arg_iter.next().unwrap();
                        result += &format!("{:x}", byte);
                        break;
                    }
                    '0' => flag = Some('0'),
                    '1'..='9' => {
                        if let Some(x) = width {
                            width = Some(x * 10 + c.to_digit(10).unwrap() as usize)
                        } else {
                            width = Some(c.to_digit(10).unwrap() as usize)
                        }
                    }
                    _ => unimplemented!("Unknown format: {}", c),
                }
            }
        } else {
            result.push(x);
        }
    }

    Ok(result)
}

pub async fn get_cur_program_id(_context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_knlGetCurProgramID()");

    Ok(1)
}
#[cfg(test)]
mod test {
    use alloc::{boxed::Box, string::String};

    use wie_util::{read_null_terminated_string_bytes, write_null_terminated_string_bytes, Result};

    use crate::{context::test::TestContext, method::MethodImpl, WIPICContext};

    use super::sprintk;

    #[futures_test::test]
    async fn test_sprintk() -> Result<()> {
        let mut context = TestContext::new();

        let sprintk = sprintk.into_body();

        let format = context.alloc_raw(10).unwrap();
        let dest = context.alloc_raw(10).unwrap();

        write_null_terminated_string_bytes(&mut context, format, "%d".as_bytes()).unwrap();
        sprintk.call(&mut context, Box::new([dest, format, 1234, 0, 0, 0, 0])).await.unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "1234");

        write_null_terminated_string_bytes(&mut context, format, "test %02d".as_bytes()).unwrap();
        sprintk.call(&mut context, Box::new([dest, format, 1, 0, 0, 0, 0])).await.unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "test 01");

        Ok(())
    }
}
