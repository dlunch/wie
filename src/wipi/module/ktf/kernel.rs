mod init;
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

pub fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32) -> anyhow::Result<init::ProgramInfo> {
    init::init(core, context, base_address, bss_size)
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
