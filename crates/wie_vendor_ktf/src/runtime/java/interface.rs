use alloc::{
    string::{String, ToString},
    vec::Vec,
};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::Backend;
use wie_base::util::{read_generic, write_generic, ByteRead};
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::{r#impl::java::lang::String as JavaString, JavaContext, JavaObjectProxy};

use crate::runtime::java::context::{JavaFullName, KtfJavaContext};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WIPIJBInterface {
    unk1: u32,
    fn_java_jump_1: u32,
    fn_java_jump_2: u32,
    fn_store_array: u32,
    fn_get_java_method: u32,
    unk4: u32,
    fn_unk4: u32,
    fn_unk5: u32,
    unk7: u32,
    unk8: u32,
    fn_unk2: u32,
    fn_register_java_string: u32,
    fn_call_native: u32,
}

pub fn get_wipi_jb_interface(core: &mut ArmCore, backend: &Backend) -> anyhow::Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_java_jump_1: core.register_function(java_jump_1, backend)?,
        fn_java_jump_2: core.register_function(java_jump_2, backend)?,
        fn_store_array: core.register_function(store_array, backend)?,
        fn_get_java_method: core.register_function(get_java_method, backend)?,
        unk4: 0,
        fn_unk4: core.register_function(jb_unk4, backend)?,
        fn_unk5: core.register_function(jb_unk5, backend)?,
        unk7: 0,
        unk8: 0,
        fn_unk2: core.register_function(jb_unk2, backend)?,
        fn_register_java_string: core.register_function(register_java_string, backend)?,
        fn_call_native: core.register_function(call_native, backend)?,
    };

    let address = Allocator::alloc(core, size_of::<WIPIJBInterface>() as u32)?;
    write_generic(core, address, interface)?;

    Ok(address)
}

pub async fn java_class_load(core: &mut ArmCore, backend: &mut Backend, ptr_target: u32, name: String) -> anyhow::Result<u32> {
    tracing::trace!("load_java_class({:#x}, {})", ptr_target, name);

    let result = KtfJavaContext::new(core, backend).load_class(ptr_target, &name).await;

    if result.is_ok() {
        Ok(0)
    } else {
        tracing::error!("load_java_class failed: {}", result.err().unwrap());

        Ok(1)
    }
}

pub async fn java_throw(_: &mut ArmCore, _: &mut Backend, error: String, a1: u32) -> anyhow::Result<u32> {
    tracing::error!("java_throw({}, {})", error, a1);

    Err(anyhow::anyhow!("Java Exception thrown {}, {:#x}", error, a1))
}

async fn get_java_method(core: &mut ArmCore, backend: &mut Backend, ptr_class: u32, ptr_fullname: u32) -> anyhow::Result<u32> {
    let fullname = JavaFullName::from_ptr(core, ptr_fullname)?;
    tracing::trace!("get_java_method({:#x}, {})", ptr_class, fullname);

    let ptr_method = KtfJavaContext::new(core, backend).get_method(ptr_class, fullname)?;

    tracing::trace!("get_java_method result {:#x}", ptr_method);

    Ok(ptr_method)
}

async fn java_jump_1(core: &mut ArmCore, _: &mut Backend, arg1: u32, address: u32) -> anyhow::Result<u32> {
    tracing::trace!("java_jump_1({:#x}, {:#x})", arg1, address);

    core.run_function::<u32>(address, &[arg1]).await
}

async fn jb_unk2(_: &mut ArmCore, _: &mut Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    tracing::warn!("stub jb_unk2({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn register_java_string(core: &mut ArmCore, backend: &mut Backend, offset: u32, length: u32) -> anyhow::Result<u32> {
    tracing::trace!("register_java_string({:#x}, {:#x})", offset, length);

    let mut cursor = offset;
    let length = if length == 0xffff_ffff {
        let length: u16 = read_generic(core, offset)?;
        cursor += 2;
        length
    } else {
        length as _
    };
    let bytes = core.read_bytes(cursor, (length * 2) as _)?;
    let bytes_u16 = bytes.chunks(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect::<Vec<_>>();
    let str = String::from_utf16(&bytes_u16)?;

    let mut context = KtfJavaContext::new(core, backend);
    let instance = JavaString::to_java_string(&mut context, &str).await?;

    Ok(instance.ptr_instance)
}

async fn jb_unk4(_: &mut ArmCore, _: &mut Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    tracing::warn!("stub jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn jb_unk5(_: &mut ArmCore, _: &mut Backend, a0: u32, a1: u32) -> anyhow::Result<u32> {
    tracing::warn!("stub jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn call_native(core: &mut ArmCore, _: &mut Backend, address: u32, ptr_data: u32) -> anyhow::Result<u32> {
    tracing::trace!("java_jump_native({:#x}, {:#x})", address, ptr_data);

    let result = core.run_function::<u32>(address, &[ptr_data]).await?;

    write_generic(core, ptr_data, result)?;
    write_generic(core, ptr_data + 4, 0u32)?;

    Ok(ptr_data)
}

async fn java_jump_2(core: &mut ArmCore, _: &mut Backend, arg1: u32, arg2: u32, address: u32) -> anyhow::Result<u32> {
    tracing::trace!("java_jump_2({:#x}, {:#x}, {:#x})", arg1, arg2, address);

    core.run_function::<u32>(address, &[arg1, arg2]).await
}

async fn store_array(core: &mut ArmCore, backend: &mut Backend, array: u32, index: u32, value: u32) -> anyhow::Result<u32> {
    tracing::trace!("store_array({:#x}, {:#x}, {:#x})", array, index, value);

    let mut context = KtfJavaContext::new(core, backend);
    context.store_array_u32(&JavaObjectProxy::new(array), index, &[value])?;

    Ok(0)
}

pub async fn java_new(core: &mut ArmCore, backend: &mut Backend, ptr_class: u32) -> anyhow::Result<u32> {
    tracing::trace!("java_new({:#x})", ptr_class);

    let instance = KtfJavaContext::new(core, backend).instantiate_from_ptr_class(ptr_class).await?;

    Ok(instance.ptr_instance)
}

pub async fn java_array_new(core: &mut ArmCore, backend: &mut Backend, element_type: u32, count: u32) -> anyhow::Result<u32> {
    tracing::trace!("java_array_new({:#x}, {:#x})", element_type, count);

    let mut java_context = KtfJavaContext::new(core, backend);

    // HACK: we don't have element type class
    let instance = if element_type > 0x100 {
        java_context.instantiate_array_from_ptr_class(element_type, count).await?
    } else {
        let element_type_name = (element_type as u8 as char).to_string();
        java_context.instantiate_array(&element_type_name, count).await?
    };

    Ok(instance.ptr_instance)
}

pub async fn java_check_cast(_: &mut ArmCore, _: &mut Backend, ptr_class: u32, ptr_instance: u32) -> anyhow::Result<u32> {
    tracing::warn!("stub java_check_cast({:#x}, {:#x})", ptr_class, ptr_instance);

    Ok(1)
}
