use alloc::{format, string::String, vec};
use core::cmp::min;

use wie_core_arm::ArmCore;
use wie_util::{write_null_terminated_string, ByteRead, ByteWrite, Result, WieError};

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
        _ => return Err(WieError::FatalError(format!("Unknown lgt stdlib import: {:#x}", function_index))),
    })
}

async fn strcpy(core: &mut ArmCore, _: &mut (), dst: u32, src: String) -> Result<()> {
    tracing::debug!("strcpy({:#x}, {})", dst, src);

    write_null_terminated_string(core, dst, &src)?;

    Ok(())
}

async fn strncpy(core: &mut ArmCore, _: &mut (), dst: u32, src: String, size: u32) -> Result<()> {
    tracing::debug!("strncpy({:#x}, {}, {:#x})", dst, src, size);

    let size_to_copy = min(size, src.len() as u32);
    let bytes = &src.as_bytes()[..size_to_copy as usize];

    core.write_bytes(dst, bytes)?;

    Ok(())
}

async fn strcmp(_core: &mut ArmCore, _: &mut (), str1: String, str2: String) -> Result<u32> {
    tracing::debug!("strcmp({}, {})", str1, str2);

    Ok(str1.cmp(&str2) as u32)
}

async fn strlen(_core: &mut ArmCore, _: &mut (), str: String) -> Result<u32> {
    tracing::debug!("strlen({})", str);

    Ok(str.len() as u32)
}

async fn atoi(_core: &mut ArmCore, _: &mut (), str: String) -> Result<u32> {
    tracing::debug!("atoi({})", str);

    Ok(str.parse().unwrap())
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

async fn unk2(_core: &mut ArmCore, _: &mut (), a0: String) -> Result<()> {
    tracing::warn!("unk2({})", a0);

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
