use alloc::{format, string::String};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use core::{
    cmp::{Ordering, min},
    iter::from_fn,
};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};
use wie_wipi_c::api::kernel;

use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

// GLOBAL_DATA_BASE + 0x101c, immediately after the LGT display properties.
const STRTOK_STATE: u32 = 0x7fff101c;

pub fn register_stdlib_svc_handler(core: &mut ArmCore, system: &System) -> Result<()> {
    async fn handle_stdlib_svc(core: &mut ArmCore, system: &mut System, id: SvcId) -> Result<()> {
        let (_, lr) = core.read_pc_lr()?;

        match id.0 {
            x if x == StdlibSvcId::Unk2 as u32 => EmulatedFunction::call(&unk2, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Sprintf as u32 => EmulatedFunction::call(&sprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Vsprintf as u32 => EmulatedFunction::call(&vsprintf, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atoi as u32 => EmulatedFunction::call(&atoi, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strtol as u32 => EmulatedFunction::call(&strtol, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Rand as u32 => EmulatedFunction::call(&rand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Srand as u32 => EmulatedFunction::call(&srand, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Strcpy as u32 => EmulatedFunction::call(&strcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncpy as u32 => EmulatedFunction::call(&strncpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcat as u32 => EmulatedFunction::call(&strcat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncat as u32 => EmulatedFunction::call(&strncat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcmp as u32 => EmulatedFunction::call(&strcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncmp as u32 => EmulatedFunction::call(&strncmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strchr as u32 => EmulatedFunction::call(&strchr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strstr as u32 => EmulatedFunction::call(&strstr, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strlen as u32 => EmulatedFunction::call(&strlen, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strtok as u32 => EmulatedFunction::call(&strtok, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcpy as u32 => EmulatedFunction::call(&memcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memmove as u32 => EmulatedFunction::call(&memmove, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcmp as u32 => EmulatedFunction::call(&memcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memset as u32 => EmulatedFunction::call(&memset, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Time as u32 => EmulatedFunction::call(&time, core, system).await?.write(core, lr),
            x if x == StdlibSvcId::Localtime as u32 => EmulatedFunction::call(&localtime, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atexit as u32 => EmulatedFunction::call(&atexit, core, &mut ()).await?.write(core, lr),
            _ => Err(WieError::FatalError(format!("Unknown lgt stdlib import: {:#x}", id.0))),
        }
    }

    core.register_svc_handler(SVC_CATEGORY_STDLIB, handle_stdlib_svc, system)
}

async fn srand(_core: &mut ArmCore, system: &mut System, seed: u32) -> Result<()> {
    tracing::debug!("srand({seed:#x})");

    system.set_random_state(seed);
    Ok(())
}

async fn rand(_core: &mut ArmCore, system: &mut System) -> Result<u32> {
    tracing::debug!("rand()");

    let state = system.random_state();
    let state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
    system.set_random_state(state);

    Ok((state >> 16) & 0x7fff)
}

#[allow(clippy::too_many_arguments)]
async fn sprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, a0: u32, a1: u32, a2: u32, a3: u32, a4: u32, a5: u32) -> Result<u32> {
    tracing::debug!("sprintf({ptr_dst:#x}, {ptr_format:#x}, {a0:#x}, {a1:#x}, {a2:#x}, {a3:#x}, {a4:#x}, {a5:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let result = kernel::sprintf(core, &format, [a0, a1, a2, a3, a4, a5].into_iter().map(Ok))?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

async fn vsprintf(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_format: u32, mut ptr_args: u32) -> Result<u32> {
    tracing::debug!("vsprintf({ptr_dst:#x}, {ptr_format:#x}, {ptr_args:#x})");

    let format = read_null_terminated_string_bytes(core, ptr_format)?;
    let args = from_fn(|| {
        let value = read_generic(core, ptr_args);
        ptr_args = ptr_args.wrapping_add(4);
        Some(value)
    });
    let result = kernel::sprintf(core, &format, args)?;
    write_null_terminated_string_bytes(core, ptr_dst, &result)?;

    Ok(result.len() as u32)
}

async fn strncpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncpy({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;

    let size_to_copy = min(size, src.len() as u32);
    let bytes = &src[..size_to_copy as usize];

    core.write_bytes(ptr_dst, bytes)?;

    Ok(())
}

const COPY_CHUNK: usize = 4096;
const STR_SCAN_CHUNK: usize = 256;

async fn memcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, len: u32) -> Result<()> {
    let mut buf = [0u8; COPY_CHUNK];
    let mut offset: u32 = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.read_bytes(ptr_src.wrapping_add(offset), &mut buf[..chunk])?;
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        offset = offset.wrapping_add(chunk as u32);
    }
    Ok(())
}

async fn memmove(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, len: u32) -> Result<u32> {
    let mut buf = [0; COPY_CHUNK];
    let mut copied = 0;
    while copied < len {
        let chunk = ((len - copied) as usize).min(COPY_CHUNK);
        let offset = if ptr_dst > ptr_src { len - copied - chunk as u32 } else { copied };
        core.read_bytes(ptr_src.wrapping_add(offset), &mut buf[..chunk])?;
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        copied += chunk as u32;
    }
    Ok(ptr_dst)
}

async fn memcmp(core: &mut ArmCore, _: &mut (), ptr_first: u32, ptr_second: u32, len: u32) -> Result<u32> {
    let mut first = [0; COPY_CHUNK];
    let mut second = [0; COPY_CHUNK];
    let mut offset = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.read_bytes(ptr_first.wrapping_add(offset), &mut first[..chunk])?;
        core.read_bytes(ptr_second.wrapping_add(offset), &mut second[..chunk])?;
        let result = first[..chunk].cmp(&second[..chunk]);
        if result != Ordering::Equal {
            return Ok(result as u32);
        }
        offset += chunk as u32;
    }
    Ok(0)
}

async fn memset(core: &mut ArmCore, _: &mut (), ptr_dst: u32, value: u32, len: u32) -> Result<()> {
    let buf = [value as u8; COPY_CHUNK];
    let mut offset: u32 = 0;
    while offset < len {
        let chunk = ((len - offset) as usize).min(COPY_CHUNK);
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..chunk])?;
        offset = offset.wrapping_add(chunk as u32);
    }
    Ok(())
}

/// R0 already holds the original destination pointer for the ARM ABI return.
async fn strcpy(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    let mut buf = [0u8; STR_SCAN_CHUNK];
    let mut offset: u32 = 0;
    loop {
        core.read_bytes(ptr_src.wrapping_add(offset), &mut buf)?;
        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            core.write_bytes(ptr_dst.wrapping_add(offset), &buf[..=pos])?;
            return Ok(());
        }
        core.write_bytes(ptr_dst.wrapping_add(offset), &buf)?;
        offset = offset.wrapping_add(STR_SCAN_CHUNK as u32);
    }
}

async fn strlen(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    let mut buf = [0u8; STR_SCAN_CHUNK];
    let mut len: u32 = 0;
    loop {
        core.read_bytes(ptr_str.wrapping_add(len), &mut buf)?;
        if let Some(pos) = buf.iter().position(|&b| b == 0) {
            return Ok(len.wrapping_add(pos as u32));
        }
        len = len.wrapping_add(STR_SCAN_CHUNK as u32);
    }
}

async fn strcat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32) -> Result<()> {
    tracing::debug!("strcat({ptr_dst:#x}, {ptr_src:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    let dst = read_null_terminated_string_bytes(core, ptr_dst)?;

    let offset = dst.len();
    write_null_terminated_string_bytes(core, ptr_dst + offset as u32, &src)?;

    Ok(())
}

async fn strcmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32) -> Result<u32> {
    tracing::debug!("strcmp({ptr_str1:#x}, {ptr_str2:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

async fn strncat(core: &mut ArmCore, _: &mut (), ptr_dst: u32, ptr_src: u32, size: u32) -> Result<u32> {
    tracing::debug!("strncat({ptr_dst:#x}, {ptr_src:#x}, {size:#x})");

    let dst_len = read_null_terminated_string_bytes(core, ptr_dst)?.len() as u32;
    let mut copied = 0;
    while copied < size {
        let byte: u8 = read_generic(core, ptr_src.wrapping_add(copied))?;
        if byte == 0 {
            break;
        }
        write_generic(core, ptr_dst.wrapping_add(dst_len).wrapping_add(copied), byte)?;
        copied += 1;
    }
    write_generic(core, ptr_dst.wrapping_add(dst_len).wrapping_add(copied), 0u8)?;

    Ok(ptr_dst)
}

async fn strncmp(core: &mut ArmCore, _: &mut (), ptr_str1: u32, ptr_str2: u32, size: u32) -> Result<u32> {
    tracing::debug!("strncmp({ptr_str1:#x}, {ptr_str2:#x}, {size:#x})");

    for offset in 0..size {
        let first: u8 = read_generic(core, ptr_str1.wrapping_add(offset))?;
        let second: u8 = read_generic(core, ptr_str2.wrapping_add(offset))?;
        if first != second || first == 0 {
            return Ok((i32::from(first) - i32::from(second)) as u32);
        }
    }

    Ok(0)
}

async fn strchr(core: &mut ArmCore, _: &mut (), mut ptr_str: u32, character: u32) -> Result<u32> {
    loop {
        let byte: u8 = read_generic(core, ptr_str)?;
        if byte == character as u8 {
            return Ok(ptr_str);
        }
        if byte == 0 {
            return Ok(0);
        }
        ptr_str = ptr_str.wrapping_add(1);
    }
}

async fn strtok(core: &mut ArmCore, _: &mut (), ptr_str: u32, ptr_delimiters: u32) -> Result<u32> {
    let mut cursor = if ptr_str == 0 { read_generic(core, STRTOK_STATE)? } else { ptr_str };
    let delimiters = read_null_terminated_string_bytes(core, ptr_delimiters)?;
    let mut token = 0;
    while cursor != 0 {
        let byte: u8 = read_generic(core, cursor)?;
        if byte == 0 {
            cursor = 0;
            break;
        }
        if delimiters.contains(&byte) {
            if token != 0 {
                write_generic(core, cursor, 0u8)?;
                cursor = cursor.wrapping_add(1);
                break;
            }
        } else if token == 0 {
            token = cursor;
        }
        cursor = cursor.wrapping_add(1);
    }
    write_generic(core, STRTOK_STATE, cursor)?;
    Ok(token)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn strtol(core: &mut ArmCore, _: &mut (), ptr_str: u32, ptr_end: u32, mut base: u32) -> Result<u32> {
    tracing::debug!("strtol({ptr_str:#x}, {ptr_end:#x}, {base})");

    if ptr_end != 0 {
        write_generic(core, ptr_end, ptr_str)?;
    }
    // shortcut: errno has no identified guest ABI; report EINVAL/ERANGE once that ABI is known.
    if base != 0 && !(2..=36).contains(&base) {
        return Ok(0);
    }
    let bytes = read_null_terminated_string_bytes(core, ptr_str)?;
    let mut offset = 0;
    while matches!(bytes.get(offset), Some(b' ' | b'\t'..=b'\r')) {
        offset += 1;
    }
    let negative = bytes.get(offset) == Some(&b'-');
    if matches!(bytes.get(offset), Some(b'+' | b'-')) {
        offset += 1;
    }
    if (base == 0 || base == 16)
        && bytes.get(offset) == Some(&b'0')
        && matches!(bytes.get(offset + 1), Some(b'x' | b'X'))
        && bytes.get(offset + 2).is_some_and(u8::is_ascii_hexdigit)
    {
        base = 16;
        offset += 2;
    } else if base == 0 {
        base = if bytes.get(offset) == Some(&b'0') { 8 } else { 10 };
    }

    let start = offset;
    let limit = if negative { 0x8000_0000 } else { 0x7fff_ffff };
    let mut value = 0u32;
    while let Some(&byte) = bytes.get(offset) {
        let Some(digit) = char::from(byte).to_digit(base) else {
            break;
        };
        value = value.saturating_mul(base).saturating_add(digit).min(limit);
        offset += 1;
    }
    if ptr_end != 0 && offset != start {
        write_generic(core, ptr_end, ptr_str.wrapping_add(offset as u32))?;
    }
    Ok(if negative { value.wrapping_neg() } else { value })
}

async fn time(core: &mut ArmCore, system: &mut System, ptr_time: u32) -> Result<u32> {
    let epoch_seconds = (system.platform().now().raw() / 1000) as u32;
    tracing::debug!("time({ptr_time:#x}) -> {epoch_seconds}");

    if ptr_time != 0 {
        write_generic(core, ptr_time, epoch_seconds)?;
    }

    Ok(epoch_seconds)
}

// TODO is this method better suit on wie_backend?
async fn localtime(core: &mut ArmCore, _: &mut (), ptr_time: u32) -> Result<u32> {
    tracing::debug!("localtime({ptr_time:#x})");

    // TODO we need static buffer
    let result = Allocator::alloc(core, 0x2c)?;
    let time: u32 = read_generic(core, ptr_time)?;

    // TODO kst only for now
    let kst = FixedOffset::east_opt(9 * 3600).unwrap();
    let dt: DateTime<FixedOffset> = kst.timestamp_opt(time as _, 0).unwrap();

    // TODO tm struct
    write_generic(core, result, dt.second() as u32)?;
    write_generic(core, result + 0x04, dt.minute() as u32)?;
    write_generic(core, result + 0x08, dt.hour() as u32)?;
    write_generic(core, result + 0x0c, dt.day() as u32)?;
    write_generic(core, result + 0x10, (dt.month() as u32) - 1)?; // months since January
    write_generic(core, result + 0x14, (dt.year() as u32) - 1900)?; // years since 1900
    write_generic(core, result + 0x18, dt.weekday().num_days_from_sunday() as u32)?; // days since Sunday
    write_generic(core, result + 0x1c, dt.ordinal() as u32)?; // days since January 1
    write_generic(core, result + 0x20, 0u32)?; // DST flag
    write_generic(core, result + 0x24, kst.local_minus_utc() as u32)?; // timezone offset in seconds
    write_generic(core, result + 0x28, 0u32)?; // timezone abbreviation ptr

    Ok(result)
}

async fn unk2(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk2({a0:#x})");

    // error exit?

    Ok(())
}

async fn atexit(_core: &mut ArmCore, _: &mut (), callback: u32) -> Result<u32> {
    tracing::warn!("stub atexit({callback:#x})");

    Ok(0)
}

async fn strstr(core: &mut ArmCore, _: &mut (), ptr_haystack: u32, ptr_needle: u32) -> Result<u32> {
    tracing::debug!("strstr({ptr_haystack:#x}, {ptr_needle:#x})");

    let haystack = read_null_terminated_string_bytes(core, ptr_haystack)?;
    let needle = read_null_terminated_string_bytes(core, ptr_needle)?;
    let position = if needle.is_empty() {
        Some(0)
    } else {
        haystack.windows(needle.len()).position(|window| window == needle)
    };

    Ok(position.map_or(0, |position| ptr_haystack + position as u32))
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, vec::Vec};

    use futures::FutureExt;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

    use super::{STRTOK_STATE, memcpy, memset, rand, register_stdlib_svc_handler, srand, strcpy, strlen};
    use crate::runtime::SVC_CATEGORY_STDLIB;

    fn stdlib_core() -> Result<ArmCore> {
        let mut core = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        Allocator::init(&mut core)?;
        core.map(0x10000, 0x10000)?;
        let mut context = core.save_context();
        context.sp = 0x18000;
        core.restore_context(&context);
        let system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        register_stdlib_svc_handler(&mut core, &system)?;
        Ok(core)
    }

    #[test]
    fn strtol_import_parses_guest_numbers_and_reports_the_end_pointer() -> Result<()> {
        let mut core = stdlib_core()?;
        let parse = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x3ffu32)?;
        for (input, base, expected, end) in [
            (b"ff80ab|".as_slice(), 16, 0xff80ab, 6),
            (b" \t\n\x0b\x0c\r-42tail", 10, -42, 9),
            (b"+0X2a!", 0, 42, 5),
            (b"0779", 0, 63, 3),
            (b"42", 0, 42, 2),
            (b"1012", 2, 5, 3),
            (b"zZ!", 36, 1295, 2),
            (b"0x", 16, 0, 1),
            (b"0xg", 0, 0, 1),
            (b"", 10, 0, 0),
            (b" -x", 10, 0, 0),
            (b"12\x80", 10, 12, 2),
            (b"2147483647", 10, i32::MAX, 10),
            (b"-2147483648", 10, i32::MIN, 11),
            (b"2147483648!", 10, i32::MAX, 10),
            (b"-99999999999999999999!", 10, i32::MIN, 21),
            (b"1", 1, 0, 0),
            (b"1", 37, 0, 0),
        ] {
            let ptr = 0x1ffff - input.len() as u32;
            core.write_bytes(ptr, input)?;
            write_generic(&mut core, 0x1ffff, 0u8)?;
            write_generic(&mut core, 0x11000, 0u32)?;
            assert_eq!(
                core.run_function::<u32>(parse, &[ptr, 0x11000, base]).now_or_never().unwrap()? as i32,
                expected,
                "{input:?}, base {base}"
            );
            assert_eq!(read_generic::<u32, _>(&core, 0x11000)?, ptr + end);
        }
        core.write_bytes(0x10000, b"-0x80\0")?;
        assert_eq!(core.run_function::<u32>(parse, &[0x10000, 0, 16]).now_or_never().unwrap()? as i32, -128);
        Ok(())
    }

    #[test]
    fn bounded_string_imports_use_guest_limits_and_arm_return_values() -> Result<()> {
        let mut core = stdlib_core()?;
        let append = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x408u32)?;
        let compare = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x40au32)?;

        // The bounded source ends at the last mapped byte without a NUL.
        core.write_bytes(0x1fffd, b"234")?;
        core.write_bytes(0x10000, b"1,\0xxxxx")?;
        let result = core.run_function::<u32>(append, &[0x10000, 0x1fffd, 3]).now_or_never().unwrap()?;
        assert_eq!(result, 0x10000);
        let mut bytes = [0; 8];
        core.read_bytes(0x10000, &mut bytes)?;
        assert_eq!(&bytes, b"1,234\0xx");

        core.write_bytes(0x11000, b"\x80\0ignored")?;
        let result = core.run_function::<u32>(append, &[0x10000, 0x11000, 9]).now_or_never().unwrap()?;
        assert_eq!(result, 0x10000);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"1,234\x80");
        core.run_function::<u32>(append, &[0x10000, 0x20000, 0]).now_or_never().unwrap()?;
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"1,234\x80");

        for (first, second, size, sign) in [
            (b"same\0x".as_slice(), b"same\0y".as_slice(), 6, 0),
            (b"prefix-a", b"prefix-b", 7, 0),
            (b"a", b"b", 1, -1),
            (b"\x80", b"\x7f", 1, 1),
            (b"a\0", b"ab", 2, -1),
        ] {
            let first_ptr = 0x20000 - first.len() as u32;
            core.write_bytes(first_ptr, first)?;
            core.write_bytes(0x11000, second)?;
            let result = core.run_function::<u32>(compare, &[first_ptr, 0x11000, size]).now_or_never().unwrap()? as i32;
            assert_eq!(result.signum(), sign);
        }
        assert_eq!(core.run_function::<u32>(compare, &[0x20000, 0x20000, 0]).now_or_never().unwrap()?, 0);

        let find = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x40bu32)?;
        core.write_bytes(0x1fffb, b"a\x80b\x80\0")?;
        for (character, expected) in [(0x180, 0x1fffc), (0, 0x1ffff), (b'c' as u32, 0)] {
            assert_eq!(core.run_function::<u32>(find, &[0x1fffb, character]).now_or_never().unwrap()?, expected);
        }
        Ok(())
    }

    #[test]
    fn tokenization_resumes_in_guest_memory_and_accepts_changing_delimiters() -> Result<()> {
        let mut core = stdlib_core()?;
        let tokenize = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x413u32)?;
        core.write_bytes(0x10000, b",one;;two:three,,\0")?;
        core.write_bytes(0x11000, b",;\0;:\0,\0\0")?;
        assert_eq!(core.run_function::<u32>(tokenize, &[0x10000, 0x11000]).now_or_never().unwrap()?, 0x10001);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10001)?, b"one");
        assert_eq!(read_generic::<u32, _>(&core, STRTOK_STATE)?, 0x10005);

        let mut resumed = core.clone();
        assert_eq!(resumed.run_function::<u32>(tokenize, &[0, 0x11003]).now_or_never().unwrap()?, 0x10006);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10006)?, b"two");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0x1000a);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x1000a)?, b"three");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0);
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11006]).now_or_never().unwrap()?, 0);

        core.write_bytes(0x10000, b"whole,string\0")?;
        assert_eq!(core.run_function::<u32>(tokenize, &[0x10000, 0x11008]).now_or_never().unwrap()?, 0x10000);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"whole,string");
        assert_eq!(core.run_function::<u32>(tokenize, &[0, 0x11008]).now_or_never().unwrap()?, 0);
        Ok(())
    }

    #[test]
    fn variadic_formatting_reads_only_consumed_guest_arguments() -> Result<()> {
        let mut core = stdlib_core()?;
        let format = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x3f9u32)?;
        core.write_bytes(0x11000, b"%u %u %u %u %u %u %u %u\0")?;
        for index in 0..8 {
            write_generic(&mut core, 0x1ffe0 + index * 4, index + 1)?;
        }
        assert_eq!(
            core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1ffe0]).now_or_never().unwrap()?,
            15
        );
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"1 2 3 4 5 6 7 8");

        core.write_bytes(0x11000, b"literal %%\0")?;
        assert_eq!(core.run_function::<u32>(format, &[0x10000, 0x11000, 0x20000]).now_or_never().unwrap()?, 9);
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"literal %");

        core.write_bytes(0x11000, b"%s:%llx\0")?;
        core.write_bytes(0x12000, b"value\0")?;
        write_generic(&mut core, 0x1fff4, 0x12000u32)?;
        write_generic(&mut core, 0x1fff8, 0x89abcdefu32)?;
        write_generic(&mut core, 0x1fffc, 0x01234567u32)?;
        core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1fff4]).now_or_never().unwrap()?;
        assert_eq!(read_null_terminated_string_bytes(&core, 0x10000)?, b"value:123456789abcdef");

        core.write_bytes(0x11000, b"%u %u\0")?;
        assert!(matches!(
            core.run_function::<u32>(format, &[0x10000, 0x11000, 0x1fffc]).now_or_never().unwrap(),
            Err(WieError::InvalidMemoryAccess(0x20000))
        ));
        Ok(())
    }

    #[test]
    fn memory_and_string_imports_preserve_guest_bytes_across_chunks() -> Result<()> {
        let mut core = stdlib_core()?;
        core.map(0x20000, 0x10000)?;
        let mut source = (0..5000).map(|index| (index % 251 + 1) as u8).collect::<Vec<_>>();
        source.push(0);
        core.write_bytes(0x10000, &source)?;
        memcpy(&mut core, &mut (), 0x20000, 0x10000, 5001).now_or_never().unwrap()?;
        let mut copied = alloc::vec![0; 5001];
        core.read_bytes(0x20000, &mut copied)?;
        assert_eq!(copied, source);
        memset(&mut core, &mut (), 0x20000, 0xff, 5001).now_or_never().unwrap()?;
        core.read_bytes(0x20000, &mut copied)?;
        assert!(copied.iter().all(|byte| *byte == 0xff));
        strcpy(&mut core, &mut (), 0x20000, 0x10000).now_or_never().unwrap()?;
        assert_eq!(strlen(&mut core, &mut (), 0x20000).now_or_never().unwrap()?, 5000);
        core.read_bytes(0x20000, &mut copied)?;
        assert_eq!(copied, source);

        let move_bytes = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x415u32)?;
        for (destination, source_ptr) in [(0x10010, 0x10000), (0x10000, 0x10010), (0x10000, 0x10000)] {
            assert_eq!(
                core.run_function::<u32>(move_bytes, &[destination, source_ptr, 5001])
                    .now_or_never()
                    .unwrap()?,
                destination
            );
            core.read_bytes(destination, &mut copied)?;
            assert_eq!(copied, source);
        }
        assert_eq!(
            core.run_function::<u32>(move_bytes, &[0x30000, 0x30000, 0]).now_or_never().unwrap()?,
            0x30000
        );

        let compare = core.make_svc_stub(SVC_CATEGORY_STDLIB, 0x416u32)?;
        assert_eq!(core.run_function::<u32>(compare, &[0x10000, 0x20000, 5001]).now_or_never().unwrap()?, 0);
        core.write_bytes(0x11000, b"\0\x80")?;
        core.write_bytes(0x21000, b"\0\x7f")?;
        for (first, second, sign) in [(0x10000, 0x20000, 1), (0x20000, 0x10000, -1)] {
            let result = core.run_function::<u32>(compare, &[first, second, 5001]).now_or_never().unwrap()? as i32;
            assert_eq!(result.signum(), sign);
        }
        assert_eq!(core.run_function::<u32>(compare, &[0x30000, 0x30000, 0]).now_or_never().unwrap()?, 0);
        Ok(())
    }

    #[test]
    fn random_state_is_shared_by_system_clones_and_process_local() -> Result<()> {
        let mut first = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        let mut second = ArmCore::new(wie_backend::Options {
            enable_gdbserver: false,
            aot: None,
            profile: None,
        })?;
        let mut first_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut second_system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let mut first_system_clone = first_system.clone();

        srand(&mut first, &mut first_system_clone, 1).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 16_838);
        assert_eq!(first_system.random_state(), 1_103_527_590);

        assert_eq!(rand(&mut second, &mut second_system).now_or_never().unwrap()?, 16_838);
        srand(&mut second, &mut second_system, 7).now_or_never().unwrap()?;
        assert_eq!(rand(&mut first, &mut first_system).now_or_never().unwrap()?, 5_758);

        Ok(())
    }
}
