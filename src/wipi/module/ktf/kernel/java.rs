use crate::core::arm::ArmCore;

use super::{
    types::{JavaClass, JavaClassDescriptor, JavaMethod},
    Context,
};

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

pub fn jb_unk1(core: &mut ArmCore, _: &Context, a0: u32, address: u32) -> u32 {
    // jump?
    log::debug!("jb_unk1({:#x}, {:#x})", a0, address);

    core.run_function(address, &[a0]).unwrap()
}

pub fn jb_unk3(_: &mut ArmCore, _: &Context, string: u32, a1: u32) -> u32 {
    // register string?
    log::debug!("jb_unk3({:#x}, {:#x})", string, a1);

    string
}
