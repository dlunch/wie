use alloc::{format, string::String, vec};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use core::cmp::min;

use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_util::{
    ByteRead, ByteWrite, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic, write_null_terminated_string_bytes,
};

use crate::runtime::{SVC_CATEGORY_STDLIB, svc_ids::StdlibSvcId};

pub fn register_stdlib_svc_handler(core: &mut ArmCore) -> Result<()> {
    async fn handle_stdlib_svc(core: &mut ArmCore, _: &mut (), id: SvcId) -> Result<()> {
        let (_, lr) = core.read_pc_lr()?;

        match id.0 {
            x if x == StdlibSvcId::Unk2 as u32 => EmulatedFunction::call(&unk2, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Atoi as u32 => EmulatedFunction::call(&atoi, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcpy as u32 => EmulatedFunction::call(&strcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strncpy as u32 => EmulatedFunction::call(&strncpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcat as u32 => EmulatedFunction::call(&strcat, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strcmp as u32 => EmulatedFunction::call(&strcmp, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk4 as u32 => EmulatedFunction::call(&unk4, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk5 as u32 => EmulatedFunction::call(&unk5, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Strlen as u32 => EmulatedFunction::call(&strlen, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memcpy as u32 => EmulatedFunction::call(&memcpy, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Memset as u32 => EmulatedFunction::call(&memset, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Localtime as u32 => EmulatedFunction::call(&localtime, core, &mut ()).await?.write(core, lr),
            x if x == StdlibSvcId::Unk3 as u32 => EmulatedFunction::call(&unk3, core, &mut ()).await?.write(core, lr),
            _ => Err(WieError::FatalError(format!("Unknown lgt stdlib import: {:#x}", id.0))),
        }
    }

    core.register_svc_handler(SVC_CATEGORY_STDLIB, handle_stdlib_svc, &())
}

async fn strcpy(core: &mut ArmCore, _: &mut (), dst: u32, ptr_src: u32) -> Result<()> {
    tracing::debug!("strcpy({dst:#x}, {ptr_src:#x})");

    let src = read_null_terminated_string_bytes(core, ptr_src)?;
    write_null_terminated_string_bytes(core, dst, &src)?;

    Ok(())
}

async fn strncpy(core: &mut ArmCore, _: &mut (), dst: u32, ptr_src: u32, size: u32) -> Result<()> {
    tracing::debug!("strncpy({dst:#x}, {ptr_src:#x}, {size:#x})");

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
    tracing::debug!("strcmp({ptr_str1:#x}, {ptr_str2:#x})");

    let str1 = read_null_terminated_string_bytes(core, ptr_str1)?;
    let str2 = read_null_terminated_string_bytes(core, ptr_str2)?;

    Ok(str1.cmp(&str2) as u32)
}

async fn strlen(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("strlen({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;

    Ok(string.len() as u32)
}

async fn atoi(core: &mut ArmCore, _: &mut (), ptr_str: u32) -> Result<u32> {
    tracing::debug!("atoi({ptr_str:#x})");

    let string = read_null_terminated_string_bytes(core, ptr_str)?;
    let string = String::from_utf8(string).unwrap();

    Ok(string.parse().unwrap_or(0))
}

async fn memcpy(core: &mut ArmCore, _: &mut (), dst: u32, src: u32, size: u32) -> Result<()> {
    tracing::debug!("memcpy({dst:#x}, {src:#x}, {size:#x})");

    let mut memory = vec![0u8; size as usize];

    core.read_bytes(src, &mut memory)?;
    core.write_bytes(dst, &memory)?;

    Ok(())
}

async fn memset(core: &mut ArmCore, _: &mut (), dst: u32, value: u32, size: u32) -> Result<()> {
    tracing::debug!("memset({dst:#x}, {value:#x}, {size:#x})");

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
    tracing::warn!("unk2({a0:#x})");

    // error exit?

    Ok(())
}

async fn unk3(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("unk3({a0:#x})");

    let _: () = core.run_function(a0, &[]).await?;

    Ok(())
}

async fn unk4(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("unk4({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(())
}

async fn unk5(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("unk5({a0:#x}, {a1:#x})");
    // strstr??

    Ok(())
}
