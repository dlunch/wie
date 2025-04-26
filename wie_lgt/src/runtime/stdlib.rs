use alloc::{format, string::String, vec};
use core::cmp::min;

use wie_core_arm::ArmCore;
use wie_util::{ByteRead, ByteWrite, Result, WieError, read_null_terminated_string_bytes, write_null_terminated_string_bytes};

pub fn get_stdlib_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x3f6 => core.register_function(unk2, &())?,
        0x3fb => core.register_function(atoi, &())?,
        0x405 => core.register_function(strcpy, &())?,
        0x406 => core.register_function(strncpy, &())?,
        0x409 => core.register_function(strcmp, &())?,
        0x40a => core.register_function(unk4, &())?,
        0x411 => core.register_function(strlen, &())?,
        0x414 => core.register_function(memcpy, &())?,
        0x418 => core.register_function(memset, &())?,
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

    Ok(string.parse().unwrap())
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
