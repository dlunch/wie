use std::mem::size_of;

use crate::core::arm::ArmCore;

use super::{interface::get_interface, java::load_java_class, misc::init_unk3, Context};

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam0 {
    pub ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam0Unk {
    pub unk: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam4 {
    pub fn_get_interface: u32,
    pub fn_unk1: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub fn_load_java_class: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1 {
    pub ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1Unk {
    pub ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1UnkUnk {
    pub unk: [u32; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WipiExe {
    pub ptr_exe_interface: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    fn_unk1: u32,
    pub fn_init: u32,
    unk3: u32,
    unk4: u32,
    fn_unk3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterface {
    pub ptr_functions: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    pub fn_init: u32,
    fn_get_default_dll: u32,
    pub fn_get_class: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

pub struct ProgramInfo {
    pub fn_init: u32,
    pub fn_get_class: u32,
}

pub fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32) -> anyhow::Result<ProgramInfo> {
    let wipi_exe = core.run_function(base_address + 1, &[bss_size])?;

    log::info!("Got wipi_exe {:#x}", wipi_exe);

    let ptr_unk_struct = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam0Unk>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_unk_struct, InitParam0Unk { unk: 0 })?;

    let ptr_param_0 = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam0>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_param_0, InitParam0 { ptr_unk_struct })?;

    let ptr_unk_struct = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam1UnkUnk>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_unk_struct, InitParam1UnkUnk { unk: [0; 8] })?;

    let ptr_unk_struct = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam1Unk>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_unk_struct, InitParam1Unk { ptr_unk_struct })?;

    let ptr_param_1 = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam1>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_param_1, InitParam1 { ptr_unk_struct })?;

    let param_4 = InitParam4 {
        fn_get_interface: core.register_function(get_interface, context)?,
        fn_unk1: 0,
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        unk5: 0,
        unk6: 0,
        fn_load_java_class: core.register_function(load_java_class, context)?,
        unk7: 0,
        unk8: 0,
        fn_unk3: core.register_function(init_unk3, context)?,
    };

    let ptr_param_4 = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam4>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(ptr_param_4, param_4)?;

    let wipi_exe = core.read::<WipiExe>(wipi_exe)?;
    let exe_interface = core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions = core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

    log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core.run_function(exe_interface_functions.fn_init, &[ptr_param_0, ptr_param_1, 0, 0, ptr_param_4])?;
    if result != 0 {
        return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
    }

    Ok(ProgramInfo {
        fn_init: wipi_exe.fn_init,
        fn_get_class: exe_interface_functions.fn_get_class,
    })
}
