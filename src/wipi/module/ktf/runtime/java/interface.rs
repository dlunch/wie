use std::mem::size_of;

use crate::core::arm::{ArmCore, EmulatedFunctionParam};

use super::{
    super::Context,
    bridge::{JavaMethodFullname, KtfJavaBridge},
};

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

impl EmulatedFunctionParam<JavaMethodFullname> for JavaMethodFullname {
    fn get(core: &mut ArmCore, pos: usize) -> JavaMethodFullname {
        let ptr = Self::read(core, pos);

        Self::from_ptr(core, ptr).unwrap()
    }
}

pub fn get_wipi_jb_interface(core: &mut ArmCore, context: &Context) -> anyhow::Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_unk1: core.register_function(jb_unk1, context)?,
        fn_unk7: core.register_function(jb_unk7, context)?,
        fn_unk8: core.register_function(jb_unk8, context)?,
        get_java_method: core.register_function(get_java_method, context)?,
        unk4: 0,
        fn_unk4: core.register_function(jb_unk4, context)?,
        fn_unk5: core.register_function(jb_unk5, context)?,
        unk7: 0,
        unk8: 0,
        fn_unk2: core.register_function(jb_unk2, context)?,
        fn_unk3: core.register_function(jb_unk3, context)?,
        fn_unk6: core.register_function(jb_unk6, context)?,
    };

    let address = context.alloc(size_of::<WIPIJBInterface>() as u32)?;
    core.write(address, interface)?;

    Ok(address)
}

pub fn java_class_load(core: &mut ArmCore, context: &Context, ptr_target: u32, name: String) -> anyhow::Result<u32> {
    log::debug!("load_java_class({:#x}, {})", ptr_target, name);

    let result = KtfJavaBridge::new(core, context).load_class(ptr_target, &name);

    if result.is_ok() {
        Ok(0)
    } else {
        log::error!("load_java_class failed: {}", result.err().unwrap());

        Ok(1)
    }
}

pub fn java_throw(core: &mut ArmCore, _: &Context, error: String, a1: u32) -> anyhow::Result<u32> {
    log::error!("java_throw({}, {})", error, a1);
    log::error!("\n{}", core.dump_regs()?);

    Ok(0)
}

fn get_java_method(core: &mut ArmCore, context: &Context, ptr_class: u32, fullname: JavaMethodFullname) -> anyhow::Result<u32> {
    log::debug!("get_java_method({:#x}, {})", ptr_class, fullname);

    let ptr_method = KtfJavaBridge::new(core, context).get_method(ptr_class, fullname)?;

    log::debug!("get_java_method result {:#x}", ptr_method);

    Ok(ptr_method)
}

fn jb_unk1(core: &mut ArmCore, _: &Context, arg1: u32, address: u32) -> anyhow::Result<u32> {
    // jump?
    log::debug!("jb_unk1({:#x}, {:#x})", arg1, address);

    core.run_function(address, &[arg1])
}

fn jb_unk2(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk2({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk3(_: &mut ArmCore, _: &Context, string: u32, a1: u32) -> anyhow::Result<u32> {
    // register string?
    log::debug!("jb_unk3({:#x}, {:#x})", string, a1);

    Ok(string)
}

fn jb_unk4(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk5(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk6(core: &mut ArmCore, _: &Context, address: u32, arg1: u32, arg2: u32) -> anyhow::Result<u32> {
    // call native function?
    log::debug!("jb_unk6({:#x}, {:#x}, {:#x})", address, arg1, arg2);

    core.run_function(address, &[arg1, arg2])
}

fn jb_unk7(core: &mut ArmCore, _: &Context, arg1: u32, arg2: u32, address: u32) -> anyhow::Result<u32> {
    // jump?
    log::debug!("jb_unk7({:#x}, {:#x}, {:#x})", arg1, arg2, address);

    core.run_function(address, &[arg1, arg2])
}

fn jb_unk8(_: &mut ArmCore, _: &Context, a0: u32, a1: u32, a2: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk8({:#x}, {:#x}, {:#x})", a0, a1, a2);

    Ok(0)
}

pub fn java_new(core: &mut ArmCore, context: &Context, ptr_class: u32) -> anyhow::Result<u32> {
    log::debug!("java_new({:#x})", ptr_class);

    let instance = KtfJavaBridge::new(core, context).instantiate_from_ptr_class(ptr_class)?;

    Ok(instance.ptr_instance)
}

pub fn java_array_new(_: &mut ArmCore, context: &Context, element_type: u32, size: u32) -> anyhow::Result<u32> {
    log::debug!("java_array_new({:#x}, {:#x})", element_type, size);

    context.alloc(size * 4)
}
