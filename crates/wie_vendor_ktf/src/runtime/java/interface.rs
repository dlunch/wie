use alloc::string::{String, ToString};
use core::mem::size_of;

use wie_backend::Backend;
use wie_base::util::write_generic;
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::JavaContextBase;

use crate::runtime::java::context::{JavaFullName, KtfJavaContext};

#[repr(C)]
#[derive(Clone, Copy)]
struct WIPIJBInterface {
    unk1: u32,
    fn_unk1: u32,
    fn_unk7: u32,
    fn_unk8: u32,
    get_java_method: u32,
    unk4: u32,
    fn_unk4: u32,
    fn_unk5: u32,
    unk7: u32,
    unk8: u32,
    fn_unk2: u32,
    fn_unk3: u32,
    fn_unk6: u32,
}

pub fn get_wipi_jb_interface(mut core: ArmCore, backend: &Backend) -> anyhow::Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_unk1: core.register_function(jb_unk1, backend)?,
        fn_unk7: core.register_function(jb_unk7, backend)?,
        fn_unk8: core.register_function(jb_unk8, backend)?,
        get_java_method: core.register_function(get_java_method, backend)?,
        unk4: 0,
        fn_unk4: core.register_function(jb_unk4, backend)?,
        fn_unk5: core.register_function(jb_unk5, backend)?,
        unk7: 0,
        unk8: 0,
        fn_unk2: core.register_function(jb_unk2, backend)?,
        fn_unk3: core.register_function(jb_unk3, backend)?,
        fn_unk6: core.register_function(jb_unk6, backend)?,
    };

    let address = Allocator::alloc(&mut core, size_of::<WIPIJBInterface>() as u32)?;
    write_generic(&mut core, address, interface)?;

    Ok(address)
}

pub fn java_class_load(core: ArmCore, backend: Backend, ptr_target: u32, name: String) -> anyhow::Result<u32> {
    log::trace!("load_java_class({:#x}, {})", ptr_target, name);

    let result = KtfJavaContext::new(core, backend).load_class(ptr_target, &name);

    if result.is_ok() {
        Ok(0)
    } else {
        log::error!("load_java_class failed: {}", result.err().unwrap());

        Ok(1)
    }
}

pub fn java_throw(core: ArmCore, _: Backend, error: String, a1: u32) -> anyhow::Result<u32> {
    log::error!("java_throw({}, {})", error, a1);
    log::error!("\n{}", core.dump_regs()?);

    Ok(0)
}

fn get_java_method(core: ArmCore, backend: Backend, ptr_class: u32, ptr_fullname: u32) -> anyhow::Result<u32> {
    let fullname = JavaFullName::from_ptr(&core, ptr_fullname)?;
    log::trace!("get_java_method({:#x}, {})", ptr_class, fullname);

    let ptr_method = KtfJavaContext::new(core, backend).get_method(ptr_class, fullname)?;

    log::trace!("get_java_method result {:#x}", ptr_method);

    Ok(ptr_method)
}

fn jb_unk1(mut core: ArmCore, _: Backend, arg1: u32, address: u32) -> anyhow::Result<u32> {
    // jump?
    log::trace!("jb_unk1 jump?({:#x}, {:#x})", arg1, address);

    core.run_function(address, &[arg1]) // TODO change to jump
}

fn jb_unk2(_: ArmCore, _: Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk2({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk3(_: ArmCore, _: Backend, string: u32, a1: u32) -> anyhow::Result<u32> {
    // register string?
    log::debug!("jb_unk3({:#x}, {:#x})", string, a1);

    Ok(string)
}

fn jb_unk4(_: ArmCore, _: Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk5(_: ArmCore, _: Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk6(mut core: ArmCore, _: Backend, address: u32, arg1: u32, arg2: u32) -> anyhow::Result<u32> {
    // call native function?
    log::trace!("jb_unk6 jump?({:#x}, {:#x}, {:#x})", address, arg1, arg2);

    core.run_function(address, &[arg1, arg2]) // TODO change to jump
}

fn jb_unk7(mut core: ArmCore, _: Backend, arg1: u32, arg2: u32, address: u32) -> anyhow::Result<u32> {
    // jump?
    log::trace!("jb_unk7 jump?({:#x}, {:#x}, {:#x})", arg1, arg2, address);

    core.run_function(address, &[arg1, arg2]) // TODO change to jump
}

fn jb_unk8(_: ArmCore, _: Backend, a0: u32, a1: u32, a2: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk8({:#x}, {:#x}, {:#x})", a0, a1, a2);

    Ok(0)
}

pub fn java_new(core: ArmCore, backend: Backend, ptr_class: u32) -> anyhow::Result<u32> {
    log::trace!("java_new({:#x})", ptr_class);

    let instance = KtfJavaContext::new(core, backend).instantiate_from_ptr_class(ptr_class)?;

    Ok(instance.ptr_instance)
}

pub fn java_array_new(core: ArmCore, backend: Backend, element_type: u32, count: u32) -> anyhow::Result<u32> {
    log::trace!("java_array_new({:#x}, {:#x})", element_type, count);

    let mut java_context = KtfJavaContext::new(core, backend);

    // HACK: we don't have element type class
    let instance = if element_type > 0x100 {
        java_context.instantiate_array_from_ptr_class(element_type, count)?
    } else {
        let element_type_name = (element_type as u8 as char).to_string();
        java_context.instantiate_array(&element_type_name, count)?
    };

    Ok(instance.ptr_instance)
}
