use alloc::string::String;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::System;
use wie_common::util::{read_generic, write_generic};
use wie_core_arm::{Allocator, ArmCore};

use crate::runtime::{
    java::{
        interface::{get_wipi_jb_interface, java_array_new, java_check_cast, java_class_load, java_new, java_throw},
        jvm_support::KtfJvmSupport,
    },
    wipi_c::interface::get_wipic_knl_interface,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam0 {
    unk: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam4 {
    fn_get_interface: u32,
    fn_java_throw: u32,
    unk1: u32,
    unk2: u32,
    fn_java_check_cast: u32,
    fn_java_new: u32,
    fn_java_array_new: u32,
    unk6: u32,
    fn_java_class_load: u32,
    unk7: u32,
    unk8: u32,
    fn_alloc: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam1 {
    ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam1Unk {
    unk: [u32; 8],
    current_java_exception_handler: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam2 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    ptr_vtables: [u32; 64],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam3 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    // java array allocation pool for primitive type
    boolean: u32,
    char: u32,
    float: u32,
    double: u32,
    byte: u32,
    short: u32,
    int: u32,
    long: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WipiExe {
    ptr_exe_interface: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    fn_unk1: u32,
    fn_init: u32,
    unk3: u32,
    unk4: u32,
    fn_unk3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExeInterface {
    ptr_functions: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    fn_init: u32,
    fn_get_default_dll: u32,
    fn_get_class: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct KtfPeb {
    pub ptr_java_context_data: u32,
    pub ptr_current_java_exception_handler: u32,
}

pub async fn start(core: &mut ArmCore, image_base: u32, bss_size: u32) -> anyhow::Result<u32> {
    core.run_function(image_base + 1, &[bss_size]).await
}

pub async fn init(core: &mut ArmCore, system: &mut System, wipi_exe: u32) -> anyhow::Result<u32> {
    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { unk: 0 })?;

    let ptr_unk_struct = Allocator::alloc(core, size_of::<InitParam1Unk>() as u32)?;
    write_generic(
        core,
        ptr_unk_struct,
        InitParam1Unk {
            unk: [0; 8],
            current_java_exception_handler: 0,
        },
    )?;

    let ptr_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    write_generic(core, ptr_param_1, InitParam1 { ptr_unk_struct })?;

    let ptr_param_2 = Allocator::alloc(core, (size_of::<InitParam2>()) as u32)?;
    write_generic(
        core,
        ptr_param_2,
        InitParam2 {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_vtables: [0; 64],
        },
    )?;

    let param_3 = InitParam3 {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        boolean: b'Z' as u32,
        char: b'C' as u32,
        float: b'F' as u32,
        double: b'D' as u32,
        byte: b'B' as u32,
        short: b'S' as u32,
        int: b'I' as u32,
        long: b'J' as u32,
    };

    let ptr_param_3 = Allocator::alloc(core, size_of::<InitParam3>() as u32)?;
    write_generic(core, ptr_param_3, param_3)?;

    let param_4 = InitParam4 {
        fn_get_interface: core.register_function(get_interface)?,
        fn_java_throw: core.register_function(java_throw)?,
        unk1: 0,
        unk2: 0,
        fn_java_check_cast: core.register_function(java_check_cast)?,
        fn_java_new: core.register_function(java_new)?,
        fn_java_array_new: core.register_function(java_array_new)?,
        unk6: 0,
        fn_java_class_load: core.register_function(java_class_load)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.register_function(alloc)?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let wipi_exe: WipiExe = read_generic(core, wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    let ptr_vtables_base = ptr_param_2 + 12;
    KtfJvmSupport::init(core, system, ptr_vtables_base, exe_interface_functions.fn_get_class, ptr_unk_struct + 32).await?;

    tracing::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_param_1, ptr_param_2, ptr_param_3, ptr_param_4],
        )
        .await?;
    anyhow::ensure!(result == 0, "Init failed with code {:#x}", result);

    Ok(wipi_exe.fn_init)
}

async fn get_interface(core: &mut ArmCore, system: &mut System, r#struct: String) -> anyhow::Result<u32> {
    tracing::trace!("get_interface({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, system),
        "WIPI_JBInterface" => get_wipi_jb_interface(core),
        _ => {
            tracing::warn!("Unknown {}", r#struct);

            Ok(0)
        }
    }
}

async fn alloc(core: &mut ArmCore, _: &mut System, a0: u32) -> anyhow::Result<u32> {
    tracing::trace!("alloc({})", a0);

    Allocator::alloc(core, a0)
}
