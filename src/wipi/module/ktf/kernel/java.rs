use std::mem::size_of;

use crate::{
    core::arm::ArmCore,
    wipi::java::{get_java_impl, JavaMethodBody},
};

use super::{
    types::{JavaClass, JavaClassDescriptor, JavaMethod},
    Context,
};

pub fn load_java_class(core: &mut ArmCore, context: &Context, ptr_target: u32, name: String) -> u32 {
    log::debug!("load_java_class({:#x}, {})", ptr_target, name);

    let r#impl = get_java_impl(&name);

    let ptr_class = context.borrow_mut().allocator.alloc(size_of::<JavaClass>() as u32).unwrap();
    core.write(
        ptr_class,
        JavaClass {
            ptr_next: ptr_class + 4,
            unk1: 0,
            ptr_descriptor: 0,
            unk2: 0,
            unk3: 0,
        },
    )
    .unwrap();

    let ptr_methods = context
        .borrow_mut()
        .allocator
        .alloc(((r#impl.methods.len() + 1) * size_of::<u32>()) as u32)
        .unwrap();

    let mut cursor = ptr_methods;
    for method in r#impl.methods {
        let ptr_name = context.borrow_mut().allocator.alloc((method.name.len() + 1) as u32).unwrap();
        core.write_raw(ptr_name, method.name.as_bytes()).unwrap();

        let ptr_method = context.borrow_mut().allocator.alloc(size_of::<JavaMethod>() as u32).unwrap();
        let fn_body = register_java_proxy(core, method.body);
        core.write(
            ptr_method,
            JavaMethod {
                fn_body,
                ptr_class,
                unk1: 0,
                ptr_name,
                unk2: 0,
                unk3: 0,
                unk4: 0,
            },
        )
        .unwrap();

        core.write(cursor, ptr_method).unwrap();
        cursor += 4;
    }

    let ptr_name = context.borrow_mut().allocator.alloc((r#impl.name.len() + 1) as u32).unwrap();
    core.write_raw(ptr_name, r#impl.name.as_bytes()).unwrap();

    let ptr_descriptor = context.borrow_mut().allocator.alloc(size_of::<JavaClassDescriptor>() as u32).unwrap();
    core.write(
        ptr_descriptor,
        JavaClassDescriptor {
            ptr_name,
            unk1: 0,
            parent_class: 0,
            ptr_methods,
            ptr_interfaces: 0,
            ptr_properties: 0,
            unk3: 0,
            unk4: 0,
            unk5: 0,
        },
    )
    .unwrap();

    core.write(ptr_class + 8, ptr_descriptor).unwrap();

    core.write(ptr_target, ptr_class).unwrap();

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
            log::debug!("get_java_method result {:#x}", ptr);

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

fn register_java_proxy(_: &mut ArmCore, _: JavaMethodBody) -> u32 {
    0
}
