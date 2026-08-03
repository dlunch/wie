use alloc::format;

use wie_core_arm::ArmCore;
use wie_util::{Result, WieError};

use crate::runtime::{SVC_CATEGORY_INIT, svc_ids::InitSvcId};

pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaInterfaceUnk0)?,
        0x06 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaDestroyRuntimeContext)?,
        0x07 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaCreateRuntimeContext)?,
        0x14 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaLinkImportedClasses)?,
        0x82 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaSetJarPath)?,
        0x83 => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaStartApplication)?,
        _ => return Err(WieError::FatalError(format!("Unknown lgt java import: {function_index:#x}"))),
    })
}

pub async fn java_unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk0({a0:#x}, {a1:#x}, {a2:#x})");

    Ok(())
}

pub async fn java_create_runtime_context(_core: &mut ArmCore, _: &mut (), generated_classes: u32, runtime_metadata: u32) -> Result<u32> {
    tracing::warn!("stub java_create_runtime_context({generated_classes:#x}, {runtime_metadata:#x})");

    Ok(generated_classes)
}

#[allow(clippy::too_many_arguments)]
pub async fn java_link_imported_classes(
    _core: &mut ArmCore,
    _: &mut (),
    imported_classes: u32,
    instance_field_imports: u32,
    static_field_imports: u32,
    virtual_method_imports: u32,
    interface_method_imports: u32,
    non_virtual_method_imports: u32,
    instance_field_slots: u32,
    static_field_slots: u32,
    virtual_method_slots: u32,
    interface_method_slots: u32,
    non_virtual_method_targets: u32,
) -> Result<()> {
    tracing::debug!(
        "java_link_imported_classes({imported_classes:#x}, {instance_field_imports:#x}, {static_field_imports:#x}, {virtual_method_imports:#x}, {interface_method_imports:#x}, {non_virtual_method_imports:#x}, {instance_field_slots:#x}, {static_field_slots:#x}, {virtual_method_slots:#x}, {interface_method_slots:#x}, {non_virtual_method_targets:#x})"
    );

    Ok(())
}

pub async fn java_set_jar_path(_core: &mut ArmCore, _: &mut (), jar_path: u32) -> Result<()> {
    tracing::warn!("stub java_set_jar_path({jar_path:#x})");

    Ok(())
}

pub async fn java_start_application(
    _core: &mut ArmCore,
    _: &mut (),
    entry_class_name: u32,
    a1: u32,
    argument_count: u32,
    arguments: u32,
) -> Result<()> {
    tracing::warn!("stub java_start_application({entry_class_name:#x}, {a1:#x}, {argument_count}, {arguments:#x})");

    Err(WieError::Unimplemented("LGT Java apps are not implemented yet".into()))
}

pub async fn java_destroy_runtime_context(_core: &mut ArmCore, _: &mut (), runtime_context: u32) -> Result<()> {
    tracing::warn!("stub java_destroy_runtime_context({runtime_context:#x})");

    Ok(())
}
