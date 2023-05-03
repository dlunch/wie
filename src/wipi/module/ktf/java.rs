use std::mem::size_of;

use super::{
    context::Context,
    types::{JavaClass, JavaClassDescriptor, JavaClassInstance, JavaMethod},
};

use crate::core::arm::ArmCore;

pub fn load_java_class(core: &mut ArmCore, context: &Context, ptr_target: u32, name: String) -> u32 {
    log::debug!("load_java_class({:#x}, {})", ptr_target, name);

    let address = context.borrow_mut().allocator.alloc(0x20).unwrap();
    let address1 = context.borrow_mut().allocator.alloc(0x20).unwrap();
    core.write(address, [0, 0, address1, 0, 0, 0, 0, 0, 0]).unwrap();
    core.write(ptr_target, address).unwrap();

    0
}

pub fn get_java_method(core: &mut ArmCore, _: &Context, ptr_class: u32, name: String) -> u32 {
    log::debug!("get_java_method({:#x}, {})", ptr_class, name);

    let class = core.read::<JavaClass>(ptr_class).unwrap();
    let descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor).unwrap();

    let mut cursor = descriptor.ptr_methods;
    loop {
        let ptr = core.read::<u32>(cursor).unwrap();
        if ptr == 0 {
            return 0;
        }

        let method = core.read::<JavaMethod>(ptr).unwrap();
        let method_name = core.read_null_terminated_string(method.ptr_name).unwrap();

        if method_name == name {
            return ptr;
        }

        cursor += 4;
    }
}

pub fn instantiate_java_class(core: &mut ArmCore, context: &Context, ptr_class: u32) -> anyhow::Result<u32> {
    let class = core.read::<JavaClass>(ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Instantiate {}", class_name);

    let instance = context
        .borrow_mut()
        .allocator
        .alloc(size_of::<JavaClassInstance>() as u32)
        .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;

    core.write(instance, JavaClassInstance { ptr_class })?;

    let method = get_java_method(core, context, ptr_class, "H()V+<init>".to_owned());
    let method = core.read::<JavaMethod>(method)?;

    log::info!("Call ctor at {:#x}", method.fn_body);

    core.run_function(method.fn_body, &[0, instance])?;

    Ok(instance)
}

pub fn call_java_method(core: &mut ArmCore, context: &Context, ptr_instance: u32, name: &str) -> anyhow::Result<u32> {
    let instance = core.read::<JavaClassInstance>(ptr_instance)?;
    let class = core.read::<JavaClass>(instance.ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Call {}::{}", class_name, name);

    let ptr_method = get_java_method(core, context, instance.ptr_class, name.to_owned());

    let method = core.read::<JavaMethod>(ptr_method)?;

    core.run_function(method.fn_body, &[0, ptr_instance])
}
