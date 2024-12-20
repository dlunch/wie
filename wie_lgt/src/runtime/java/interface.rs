use alloc::format;

use wie_core_arm::ArmCore;
use wie_util::{Result, WieError};

pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.register_function(java_unk0, &())?,
        0x06 => core.register_function(java_unk12, &())?,
        0x07 => core.register_function(java_unk5, &())?,
        0x14 => core.register_function(java_unk6, &())?,
        0x82 => core.register_function(java_unk9, &())?,
        0x83 => core.register_function(java_unk11, &())?,
        _ => return Err(WieError::FatalError(format!("Unknown lgt java import: {:#x}", function_index))),
    })
}

async fn java_unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk0({:#x}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk5(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("java_unk5({:#x}, {:#x})", a0, a1);

    // a0: class list

    Ok(())
}

async fn java_unk6(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk6({:#x}, {:#x}, {:#x})", a0, a1, a2);

    // a0: importing classes, a1: fields? a2: static fields? a3: methods?

    Ok(())
}

async fn java_unk9(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("java_unk9({:#x})", a0);

    Ok(())
}

async fn java_unk11(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("java_unk11({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    // invoke static? used to be called with org/kwis/msp/lcdui/Main

    Err(WieError::Unimplemented("Java apps are not implemented yet".into()))
}

async fn java_unk12(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("java_unk12({:#x})", a0);

    Ok(())
}
