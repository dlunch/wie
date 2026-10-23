mod sprintf;

use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec::Vec,
};
use core::iter;

use bytemuck::{Pod, Zeroable};

use wipi_types::wipic::{WIPICIndirectPtr, WIPICWord};

use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

use crate::{WIPICResult, context::WIPICContext, globals::PENDING_TIMERS_HEAD, method::MethodBody};

pub use self::sprintf::sprintf;

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICTimer {
    fn_callback: WIPICWord,
}

#[repr(C, packed)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PendingTimer {
    next: WIPICWord,
    ptr_timer: WIPICWord,
    fn_callback: WIPICWord,
    param: WIPICWord,
}

fn find_timer(context: &dyn WIPICContext, ptr_timer: WIPICWord) -> Result<(WIPICWord, WIPICWord)> {
    let mut link = PENDING_TIMERS_HEAD;
    loop {
        let ptr: WIPICWord = read_generic(context, link)?;
        if ptr == 0 || read_generic::<PendingTimer, _>(context, ptr)?.ptr_timer == ptr_timer {
            return Ok((link, ptr));
        }
        link = ptr; // The first word of each entry is the next link.
    }
}

fn cancel_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord) -> Result<()> {
    let (link, ptr) = find_timer(context, ptr_timer)?;
    if ptr != 0 {
        let mut timer: PendingTimer = read_generic(context, ptr)?;
        write_generic(context, link, timer.next)?;
        // The queued callback owns this allocation until it is dispatched.
        timer.ptr_timer = 0;
        write_generic(context, ptr, timer)?;
    }
    Ok(())
}

pub async fn current_time(context: &mut dyn WIPICContext) -> Result<u64> {
    tracing::debug!("MC_knlCurrentTime()");

    Ok(context.system().platform().now().raw())
}

pub async fn get_system_property(context: &mut dyn WIPICContext, ptr_id: WIPICWord, p_out: WIPICWord, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetSystemProperty({ptr_id:#x}, {p_out:#x}, {buf_size:#x})");

    let id_bytes = read_null_terminated_string_bytes(context, ptr_id)?;
    let id = encoding_rs::EUC_KR.decode(&id_bytes).0;

    let value = match id.as_ref() {
        "RSSILEVEL" => "30",
        "BATTERYLEVEL" => "100",
        "PHONEMODEL" => "Emulator",
        "PHONENUMBER" => "", // setting this causes some apps to fail authentication
        "MIN" => "01000000000",
        "ANNUN_CALL" => "0",
        "ANNUN_SMS" => "0",
        "ANNUN_SILENT" => "0",
        "ANNUN_ALARM" => "0",
        "ANNUN_SECURITY" => "0",
        "CURRENTCH" => "0",
        "AIRPLANE_MODE" => "0",
        "ROAMING_AREA" => "0",
        "DS_LOCK" => "0",
        _ => {
            tracing::warn!("unknown system property id: {id}");
            return Ok(-9); // M_E_INVALID
        }
    };

    let bytes = value.as_bytes();
    if bytes.len() + 1 > buf_size as usize {
        return Ok(-18); // M_E_SHORTBUF
    }

    write_null_terminated_string_bytes(context, p_out, value.as_bytes())?;

    Ok(0)
}

pub async fn set_system_property(_context: &mut dyn WIPICContext, ptr_id: WIPICWord, ptr_value: WIPICWord) -> Result<()> {
    tracing::warn!("stub MC_knlSetSystemProperty({ptr_id:#x}, {ptr_value:#x})");

    Ok(())
}

pub async fn def_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord, fn_callback: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlDefTimer({ptr_timer:#x}, {fn_callback:#x})");

    cancel_timer(context, ptr_timer)?;
    let timer = WIPICTimer { fn_callback };

    write_generic(context, ptr_timer, timer)?;

    Ok(())
}

pub async fn set_timer(
    context: &mut dyn WIPICContext,
    ptr_timer: WIPICWord,
    timeout_low: WIPICWord,
    timeout_high: WIPICWord,
    param: WIPICWord,
) -> Result<i32> {
    tracing::debug!("MC_knlSetTimer({ptr_timer:#x}, {timeout_low:#x}, {timeout_high:#x}, {param:#x})");

    struct TimerCallback {
        pending: WIPICWord,
    }

    #[async_trait::async_trait]
    impl MethodBody<WieError> for TimerCallback {
        #[tracing::instrument(name = "timer", skip_all)]
        async fn call(&self, context: &mut dyn WIPICContext, _: Box<[WIPICWord]>) -> Result<WIPICResult> {
            let timer: PendingTimer = read_generic(context, self.pending)?;
            if timer.ptr_timer != 0 {
                cancel_timer(context, timer.ptr_timer)?;
            }
            context.free_raw(self.pending, size_of::<PendingTimer>() as _)?;
            if timer.ptr_timer != 0 {
                context.call_function(timer.fn_callback, &[timer.ptr_timer, timer.param]).await?;
            }

            Ok(WIPICResult { results: Vec::new() })
        }
    }

    let now = context.system().platform().now();
    let timeout = (((timeout_high as u64) << 32) | (timeout_low as u64)) as _;
    if find_timer(context, ptr_timer)?.1 != 0 {
        return Ok(-5); // M_E_EXIST
    }
    let timer: WIPICTimer = read_generic(context, ptr_timer)?;
    let pending = context.alloc_raw(size_of::<PendingTimer>() as _)?;
    let entry = PendingTimer {
        next: read_generic(context, PENDING_TIMERS_HEAD)?,
        ptr_timer,
        fn_callback: timer.fn_callback,
        param,
    };
    write_generic(context, pending, entry)?;
    write_generic(context, PENDING_TIMERS_HEAD, pending)?;

    context.set_timer(now + timeout, Box::new(TimerCallback { pending }));

    Ok(0)
}

pub async fn unset_timer(context: &mut dyn WIPICContext, ptr_timer: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlUnsetTimer({ptr_timer:#x})");

    cancel_timer(context, ptr_timer)
}

pub async fn alloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlAlloc({size:#x})");

    if size == 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    context.alloc(size)
}

pub async fn calloc(context: &mut dyn WIPICContext, size: WIPICWord) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlCalloc({size:#x})");

    if size == 0 {
        return Ok(WIPICIndirectPtr(0));
    }

    let memory = context.alloc(size)?;

    let zero = iter::repeat_n(0, size as _).collect::<Vec<_>>();
    context.write_bytes(context.data_ptr(memory)?, &zero)?;

    Ok(memory)
}

pub async fn free(context: &mut dyn WIPICContext, memory: WIPICIndirectPtr) -> Result<WIPICIndirectPtr> {
    tracing::debug!("MC_knlFree({:#x})", memory.0);

    if memory.0 == 0 {
        return Ok(memory);
    }

    context.free(memory)?;

    Ok(memory)
}

pub async fn get_resource_id(context: &mut dyn WIPICContext, ptr_name: WIPICWord, ptr_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResourceID({ptr_name:#x}, {ptr_size:#x})");

    let raw_name = read_null_terminated_string_bytes(context, ptr_name)?;
    let name = encoding_rs::EUC_KR.decode(&raw_name).0;
    tracing::debug!("  resource name: {name}");

    let size = context.get_resource_size(&name).await?;

    if size.is_none() {
        if ptr_size != 0 {
            write_generic(context, ptr_size, 0u32)?;
        }
        return Ok(-12); // M_E_NOENT
    }

    let size = size.unwrap();

    let name_bytes = name.as_bytes();
    let handle_size = name_bytes
        .len()
        .checked_add(1)
        .ok_or_else(|| WieError::FatalError("Resource name too long".to_string()))?;
    let ptr_handle = context.alloc_raw(handle_size as _)?;
    write_null_terminated_string_bytes(context, ptr_handle, name_bytes)?;
    write_generic(context, ptr_size, size as u32)?;

    Ok(ptr_handle as _)
}

pub async fn get_resource(context: &mut dyn WIPICContext, id: i32, buf: WIPICIndirectPtr, buf_size: WIPICWord) -> Result<i32> {
    tracing::debug!("MC_knlGetResource({id}, {:#x}, {buf_size})", buf.0);

    if id < 0 {
        return Ok(-9); // M_E_INVALID
    }

    let name_bytes = read_null_terminated_string_bytes(context, id as _)?;
    // the handle was written by get_resource_id as utf-8, not guest-encoded
    let name = String::from_utf8_lossy(&name_bytes);

    let data = context.read_resource(&name).await?;

    if data.len() as u32 > buf_size {
        return Ok(-1);
    }

    context.write_bytes(context.data_ptr(buf)?, &data)?;

    Ok(0)
}

pub async fn printk(context: &mut dyn WIPICContext, ptr_format: WIPICWord, a0: WIPICWord, a1: WIPICWord, a2: WIPICWord, a3: WIPICWord) -> Result<()> {
    tracing::debug!("MC_knlPrintk({ptr_format:#x}, {a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let result = sprintf(context, &format_string, [a0, a1, a2, a3].into_iter().map(Ok))?;
    let result = encoding_rs::EUC_KR.decode(&result).0;

    context.system().platform().write_stdout(result.as_bytes());

    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub async fn sprintk(
    context: &mut dyn WIPICContext,
    dest: WIPICWord,
    ptr_format: WIPICWord,
    a0: WIPICWord,
    a1: WIPICWord,
    a2: WIPICWord,
    a3: WIPICWord,
    a4: WIPICWord,
    a5: WIPICWord,
) -> Result<WIPICWord> {
    tracing::debug!("MC_knlSprintk({dest:#x}, {ptr_format:#x}, {a1}, {a2}, {a3}, {a4}, {a5})",);

    let format_string = read_null_terminated_string_bytes(context, ptr_format)?;
    let result = sprintf(context, &format_string, [a0, a1, a2, a3, a4, a5].into_iter().map(Ok))?;

    write_null_terminated_string_bytes(context, dest, &result)?;

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

pub async fn exit(context: &mut dyn WIPICContext, code: i32) -> Result<()> {
    tracing::debug!("MC_knlExit({code})");

    context.system().platform().exit();

    Ok(())
}

pub async fn execute(_context: &mut dyn WIPICContext, name: WIPICWord, parameter_count: i32) -> Result<i32> {
    tracing::warn!("stub MC_knlExecute({name:#x}, {parameter_count})");
    Ok(-9) // M_E_INVALID: the execution name does not identify an installed program.
}

pub async fn get_cur_program_id(_context: &mut dyn WIPICContext) -> Result<WIPICWord> {
    tracing::warn!("stub MC_knlGetCurProgramID()");

    Ok(1)
}

pub async fn get_access_level(_context: &mut dyn WIPICContext) -> Result<i32> {
    tracing::warn!("stub MC_knlGetAccessLevel()");
    Ok(0)
}

pub async fn get_program_name(context: &mut dyn WIPICContext, name_buf: WIPICWord, buf_size: i32) -> Result<i32> {
    tracing::debug!("MC_knlGetProgramName({name_buf:#x}, {buf_size})");

    let aid = context.system().aid().to_string();

    if buf_size < aid.len() as i32 + 1 {
        return Ok(-18); // M_E_SHORTBUF
    }

    let aid_bytes = aid.as_bytes();
    context.write_bytes(name_buf, aid_bytes)?;
    context.write_bytes(name_buf + aid_bytes.len() as u32, &[0])?;

    Ok(0)
}

#[cfg(test)]
mod test {
    use alloc::{boxed::Box, string::String, sync::Arc, vec};

    use test_utils::{TestClock, TestPlatform};

    use wie_backend::{DefaultTaskRunner, System};
    use wie_util::{ByteRead, ByteWrite, Result, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes};

    use crate::{WIPICContext, context::test::TestContext, method::MethodImpl};

    use super::{alloc, calloc, def_timer, free, get_resource, get_resource_id, get_system_property, set_timer, sprintk, unset_timer};

    #[futures_test::test]
    async fn timers_cancel_and_rearm_without_stale_callbacks() -> Result<()> {
        async fn timer_callback(context: &mut dyn WIPICContext, timer: u32, param: u32) -> Result<u32> {
            if param == 3 {
                assert_eq!(set_timer(context, timer, 10, 0, 4).await?, 0);
            }
            Ok(0)
        }

        let clock = TestClock::new();
        let system = System::new(Box::new(TestPlatform::with_clock(clock.clone())), "", "", DefaultTaskRunner);
        let mut context = TestContext::with_system(system);
        let callback = 0x1234;
        context.functions.push((callback, Arc::new(timer_callback.into_body())));
        let timer = context.alloc_raw(8)?;
        let other_timer = context.alloc_raw(8)?;
        for timer in [timer, other_timer] {
            write_generic(&mut context, timer + 4, 0x12345678u32)?;
            def_timer(&mut context, timer, callback).await?;
        }
        for rearm in [false, true] {
            assert_eq!(set_timer(&mut context, timer, 10, 0, 1).await?, 0);
            let (_, stale) = context.timers.pop_front().unwrap();
            for _ in 0..2 {
                unset_timer(&mut context, timer).await?;
            }
            if rearm {
                assert_eq!(set_timer(&mut context, timer, 10, 0, 2).await?, 0);
            }
            clock.advance(10);
            stale.call(&mut context, Box::new([])).await?;
            assert!(context.function_calls.is_empty(), "A cancelled reservation must not invoke the callback");
        }
        assert_eq!(set_timer(&mut context, other_timer, 10, 0, 99).await?, 0);

        def_timer(&mut context, timer, callback).await?;
        assert_eq!(set_timer(&mut context, timer, 10, 0, 3).await?, 0);
        assert_eq!(set_timer(&mut context, timer, 10, 0, 77).await?, -5);
        while let Some((due, callback)) = context.timers.pop_front() {
            clock.set(due.raw());
            callback.call(&mut context, Box::new([])).await?;
        }
        assert_eq!(
            context.function_calls,
            vec![(callback, vec![other_timer, 99]), (callback, vec![timer, 3]), (callback, vec![timer, 4])]
        );
        for timer in [timer, other_timer] {
            assert_eq!(read_generic::<u32, _>(&context, timer + 4)?, 0x12345678);
        }
        Ok(())
    }

    #[futures_test::test]
    async fn test_sprintk() -> Result<()> {
        let mut context = TestContext::new();

        let sprintk = sprintk.into_body();

        let format = context.alloc_raw(10).unwrap();
        let dest = context.alloc_raw(10).unwrap();

        write_null_terminated_string_bytes(&mut context, format, "%d".as_bytes()).unwrap();
        sprintk
            .call(&mut context, Box::new([dest, format, 1234, 0, 0, 0, 0, 0, 0, 0]))
            .await
            .unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "1234");

        write_null_terminated_string_bytes(&mut context, format, "test %02d".as_bytes()).unwrap();
        sprintk
            .call(&mut context, Box::new([dest, format, 1, 0, 0, 0, 0, 0, 0, 0]))
            .await
            .unwrap();
        let result = read_null_terminated_string_bytes(&context, dest).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "test 01");

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_system_property_min() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        write_null_terminated_string_bytes(&mut context, id, b"MIN").unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), 0);
        let result = read_null_terminated_string_bytes(&context, out).unwrap();
        assert_eq!(String::from_utf8(result).unwrap(), "01000000000");

        Ok(())
    }

    #[futures_test::test]
    async fn test_zero_size_memory_returns_null() -> Result<()> {
        let mut context = TestContext::new();

        assert_eq!(alloc(&mut context, 0).await.unwrap().0, 0);
        assert_eq!(calloc(&mut context, 0).await.unwrap().0, 0);
        assert_eq!(free(&mut context, wipi_types::wipic::WIPICIndirectPtr(0)).await.unwrap().0, 0);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_system_property_non_utf8() -> Result<()> {
        let mut context = TestContext::new();
        let id = context.alloc_raw(16).unwrap();
        let out = context.alloc_raw(16).unwrap();

        // EUC-KR "한글"
        write_null_terminated_string_bytes(&mut context, id, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        assert_eq!(get_system_property(&mut context, id, out, 16).await.unwrap(), -9);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_resource_id_non_utf8() -> Result<()> {
        let mut context = TestContext::new();
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        assert_eq!(get_resource_id(&mut context, name, size).await.unwrap(), -12);

        Ok(())
    }

    #[futures_test::test]
    async fn test_get_resource_euc_kr_roundtrip() -> Result<()> {
        let data = [1u8, 2, 3, 4];
        let mut context = TestContext::new().with_resource("한글", &data);
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, &[0xc7, 0xd1, 0xb1, 0xdb]).unwrap();

        let handle = get_resource_id(&mut context, name, size).await.unwrap();
        assert!(handle >= 0);

        let mut size_bytes = [0; 4];
        context.read_bytes(size, &mut size_bytes).unwrap();
        assert_eq!(u32::from_le_bytes(size_bytes), 4);

        let buf = context.alloc(4).unwrap();
        assert_eq!(get_resource(&mut context, handle, buf, 4).await.unwrap(), 0);

        let mut result = [0; 4];
        context.read_bytes(context.data_ptr(buf).unwrap(), &mut result).unwrap();
        assert_eq!(result, data);

        Ok(())
    }

    #[futures_test::test]
    async fn test_missing_resource_clears_size() -> Result<()> {
        let mut context = TestContext::new();
        let name = context.alloc_raw(16).unwrap();
        let size = context.alloc_raw(4).unwrap();

        write_null_terminated_string_bytes(&mut context, name, b"missing").unwrap();
        context.write_bytes(size, &[0xff; 4]).unwrap();

        assert_eq!(get_resource_id(&mut context, name, size).await.unwrap(), -12);
        let mut result = [0; 4];
        context.read_bytes(size, &mut result).unwrap();
        assert_eq!(u32::from_le_bytes(result), 0);

        Ok(())
    }
}
