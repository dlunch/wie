use alloc::{format, string::String, vec};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use core::cmp::min;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};

pub fn get_stdlib_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x3f6 => core.register_function(unk2, &())?,
        0x3fb => core.register_function(atoi, &())?,
        0x405 => core.register_function(strcpy, &())?,
        0x406 => core.register_function(strncpy, &())?,
        0x407 => core.register_function(strcat, &())?,
        0x409 => core.register_function(strcmp, &())?,
        0x40a => core.register_function(unk4, &())?,
        0x410 => core.register_function(unk5, &())?,
        0x411 => core.register_function(strlen, &())?,
        0x414 => core.register_function(memcpy, &())?,
        0x418 => core.register_function(memset, &())?,
        0x420 => core.register_function(localtime, &())?,
        0x424 => core.register_function(unk3, &())?,
        _ => return Err(WieError::FatalError(format!("Unknown lgt stdlib import: {function_index:#x}"))),
    })
}

async fn strcpy(core: &mut ArmCore, _: &mut (), dst: u32, ptr_src: u32) -> Result<()> {
    tracing::debug!("strcpy({:#x}, {:#x})", dst, ptr_src);

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    write_null_terminated_string_bytes(core, dst, &src)?;

    Ok(())
}

async fn strncpy(core: &mut ArmCore, _: &mut (), dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncpy({:#x}, {:#x}, {:#x})", dst, ptr_src, size);

    let src = read_null_terminated_string_bytes(core, ptr_src)?;

    let size_to_copy = min(size, src.len() as u32);
    let bytes = &src[..size_to_copy as usize];

    core.write_bytes(dst, bytes)?;

    Ok(())
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
    tracing::debug!("strcmp({:#x}, {:#x})", ptr_str1, ptr_str2);

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

async fn strlen(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("strlen({:#x})", ptr_str);

    let string = read_null_terminated_string_bytes(core, ptr_str)?;

    Ok(string.len() as u32)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({:#x})", ptr_str);

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn memcpy(core: &mut ArmCore, _: &mut (), dst: u32, src: u32, size: u32) -> Result<()> {
    tracing::debug!("memcpy({:#x}, {:#x}, {:#x})", dst, src, size);

    let mut memory = vec![0u8; size as usize];

    core.read_bytes(src, &mut memory)?;
    core.write_bytes(dst, &memory)?;

    Ok(())
}

async fn memset(core: &mut ArmCore, _: &mut (), dst: u32, value: u32, size: u32) -> Result<()> {
    tracing::debug!("memset({:#x}, {:#x}, {:#x})", dst, value, size);

    let memory = vec![value as u8; size as usize];
    core.write_bytes(dst, &memory)?;

    Ok(())
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
    tracing::warn!("unk2({:#x})", a0);

    // error exit?

    Ok(())
}

async fn unk3(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk3({:#x})", a0);

    let _: () = core.run_function(a0, &[]).await?;

    Ok(())
}

async fn unk4(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("unk4({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    Ok(())
}

async fn unk5(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk5({a0:#x}, {a1:#x})");
    // strstr??

    Ok(())
}
