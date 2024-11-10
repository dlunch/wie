use alloc::{format, string::String};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use jvm::Jvm;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, read_null_terminated_string_bytes, write_generic, Result, WieError};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{
        java::interface::{get_wipi_jb_interface, java_array_new, java_array_store_check_object_type, java_class_load, java_new, java_throw},
        wipi_c::interface::get_wipic_knl_interface,
    },
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
    fn_java_array_store_check_object_type: u32,
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
    ptr_jvm_exception_context: u32,
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

pub async fn load_native(
    core: &mut ArmCore,
    system: &mut System,
    jvm: &Jvm,
    filename: &str,
    data: &[u8],
    ptr_jvm_context: u32,
    ptr_jvm_exception_context: u32,
) -> Result<u32> {
    let bss_start = filename.find("client.bin").unwrap() + 10;
    let bss_size = filename[bss_start..].parse::<u32>().unwrap();

    core.load(data, IMAGE_BASE, data.len() + bss_size as usize)?;

    tracing::debug!("Loaded at {:#x}, size {:#x}, bss {:#x}", IMAGE_BASE, data.len(), bss_size);

    let wipi_exe = core.run_function(IMAGE_BASE + 1, &[bss_size]).await?;
    tracing::debug!("Got wipi_exe {:#x}", wipi_exe);

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { unk: 0 })?;

    let ptr_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    write_generic(core, ptr_param_1, InitParam1 { ptr_jvm_exception_context })?;

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
        fn_get_interface: core.register_function(get_interface, &(system.clone(), jvm.clone()))?,
        fn_java_throw: core.register_function(java_throw, jvm)?,
        unk1: 0,
        unk2: 0,
        fn_java_array_store_check_object_type: core.register_function(java_array_store_check_object_type, jvm)?,
        fn_java_new: core.register_function(java_new, jvm)?,
        fn_java_array_new: core.register_function(java_array_new, jvm)?,
        unk6: 0,
        fn_java_class_load: core.register_function(java_class_load, jvm)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.register_function(alloc, &())?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let wipi_exe: WipiExe = read_generic(core, wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    tracing::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_param_1, ptr_jvm_context, ptr_param_3, ptr_param_4],
        )
        .await?;

    if result != 0 {
        return Err(WieError::FatalError(format!("Init failed with code {:#x}", result)));
    }

    let result = core.run_function::<u32>(wipi_exe.fn_init, &[]).await?;

    if result != 0 {
        return Err(WieError::FatalError(format!("wipi init failed with code {:#x}", result)));
    }

    Ok(exe_interface_functions.fn_get_class)
}

async fn get_interface(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), ptr_name: u32) -> Result<u32> {
    tracing::trace!("get_interface({:#x})", ptr_name);

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?).unwrap();

    match name.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, system, jvm),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, jvm),
        _ => {
            tracing::warn!("Unknown {}", name);

            Ok(0)
        }
    }
}

async fn alloc(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<u32> {
    tracing::trace!("alloc({})", a0);

    Allocator::alloc(core, a0)
}
