mod interface;
mod java;
mod misc;
mod types;

use std::mem::size_of;

use crate::{
    core::arm::ArmCore,
    wipi::module::ktf::kernel::types::{JavaClass, JavaClassDescriptor, JavaClassInstance, JavaMethod},
};

use super::context::Context;

use self::{
    interface::get_interface,
    java::load_java_class,
    misc::init_unk3,
    types::{ExeInterface, ExeInterfaceFunctions, InitParam4, WipiExe},
};

pub struct ProgramInfo {
    pub fn_init: u32,
    pub fn_get_class: u32,
}

pub fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32) -> anyhow::Result<ProgramInfo> {
    let wipi_exe = core.run_function(base_address + 1, &[bss_size])?;

    log::info!("Got wipi_exe {:#x}", wipi_exe);

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

    let param4_addr = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<InitParam4>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
    core.write(param4_addr, param_4)?;

    let wipi_exe = core.read::<WipiExe>(wipi_exe)?;
    let exe_interface = core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions = core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

    log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core.run_function(exe_interface_functions.fn_init, &[0, 0, 0, 0, param4_addr])?;
    if result != 0 {
        return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
    }

    Ok(ProgramInfo {
        fn_init: wipi_exe.fn_init,
        fn_get_class: exe_interface_functions.fn_get_class,
    })
}

pub fn instantiate_java_class(core: &mut ArmCore, context: &Context, ptr_class: u32) -> anyhow::Result<u32> {
    let class = core.read::<JavaClass>(ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Instantiate {}", class_name);

    let ptr_instance = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<JavaClassInstance>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;

    core.write(ptr_instance, JavaClassInstance { ptr_class })?;

    call_java_method(core, context, ptr_instance, "H()V+<init>")?;

    Ok(ptr_instance)
}

pub fn call_java_method(core: &mut ArmCore, context: &Context, ptr_instance: u32, name: &str) -> anyhow::Result<u32> {
    let instance = core.read::<JavaClassInstance>(ptr_instance)?;
    let class = core.read::<JavaClass>(instance.ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Call {}::{}", class_name, name);

    let ptr_method = java::get_java_method(core, context, instance.ptr_class, name.to_owned());

    let method = core.read::<JavaMethod>(ptr_method)?;

    core.run_function(method.fn_body, &[0, ptr_instance])
}
