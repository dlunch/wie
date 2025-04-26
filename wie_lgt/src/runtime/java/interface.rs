use alloc::format;

use wie_core_arm::ArmCore;
use wie_util::{Result, WieError};

pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.register_function(java_unk0, &())?,
        0x06 => core.register_function(java_unk12, &())?,
        0x07 => core.register_function(java_unk5, &())?,
        0x14 => core.register_function(java_load_classes, &())?,
        0x82 => core.register_function(java_unk9, &())?,
        0x83 => core.register_function(java_unk11, &())?,
        _ => return Err(WieError::FatalError(format!("Unknown lgt java import: {function_index:#x}"))),
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

#[allow(clippy::too_many_arguments)]
async fn java_load_classes(
    _core: &mut ArmCore,
    _: &mut (),
    classes: u32,
    fields: u32,
    static_fields: u32,
    virtual_methods: u32,
    a4: u32,
    static_methods: u32,
    field_offsets: u32,
    static_field_offsets: u32,
    virtual_method_offsets: u32,
    a9: u32,
    static_method_offsets: u32,
) -> Result<()> {
    tracing::debug!(
        "java_load_classes({:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x}, {:#x})",
        classes,
        fields,
        static_fields,
        virtual_methods,
        a4,
        static_methods,
        field_offsets,
        static_field_offsets,
        virtual_method_offsets,
        a9,
        static_method_offsets,
    );

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
