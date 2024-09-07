use alloc::{
    boxed::Box,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::size_of;

use jvm::{runtime::JavaLangString, Jvm};

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore, ArmCoreResult};
use wie_util::{read_generic, write_generic, ByteRead};

use crate::runtime::{java::jvm_support::KtfJvmSupport, RuntimeResult};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WIPIJBInterface {
    unk1: u32,
    fn_java_jump_1: u32,
    fn_java_jump_2: u32,
    fn_java_jump_3: u32,
    fn_get_java_method: u32,
    fn_get_static_field: u32,
    fn_unk4: u32,
    fn_unk5: u32,
    fn_unk7: u32,
    fn_unk8: u32,
    fn_register_class: u32,
    fn_register_java_string: u32,
    fn_call_native: u32,
}

pub fn get_wipi_jb_interface(core: &mut ArmCore, jvm: &Jvm) -> ArmCoreResult<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_java_jump_1: core.register_function(java_jump_1, jvm)?,
        fn_java_jump_2: core.register_function(java_jump_2, jvm)?,
        fn_java_jump_3: core.register_function(java_jump_3, jvm)?,
        fn_get_java_method: core.register_function(get_java_method, jvm)?,
        fn_get_static_field: core.register_function(get_static_field, jvm)?,
        fn_unk4: core.register_function(jb_unk4, jvm)?,
        fn_unk5: core.register_function(jb_unk5, jvm)?,
        fn_unk7: core.register_function(jb_unk7, jvm)?,
        fn_unk8: core.register_function(jb_unk8, jvm)?,
        fn_register_class: core.register_function(register_class, jvm)?,
        fn_register_java_string: core.register_function(register_java_string, jvm)?,
        fn_call_native: core.register_function(call_native, jvm)?,
    };

    let address = Allocator::alloc(core, size_of::<WIPIJBInterface>() as u32)?;
    write_generic(core, address, interface)?;

    Ok(address)
}

pub async fn java_class_load(core: &mut ArmCore, jvm: &mut Jvm, ptr_target: u32, name: String) -> RuntimeResult<u32> {
    tracing::trace!("load_java_class({:#x}, {})", ptr_target, name);

    let class = jvm.resolve_class(&name).await;

    if let Ok(x) = class {
        let raw = KtfJvmSupport::class_definition_raw(&*x.definition)?;
        write_generic(core, ptr_target, raw)?;

        Ok(0)
    } else {
        tracing::error!("load_java_class({}) failed", name);

        Ok(1)
    }
}

pub async fn java_throw(_: &mut ArmCore, _jvm: &mut Jvm, error: String, a1: u32) -> RuntimeResult<u32> {
    tracing::error!("java_throw({}, {})", error, a1);

    anyhow::bail!("Java Exception thrown {}, {:#x}", error, a1)
}

async fn get_java_method(core: &mut ArmCore, _jvm: &mut Jvm, ptr_class: u32, ptr_fullname: u32) -> RuntimeResult<u32> {
    let fullname = KtfJvmSupport::read_name(core, ptr_fullname)?;

    tracing::trace!("get_java_method({:#x}, {})", ptr_class, fullname);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let method = class.method(&fullname.name, &fullname.descriptor)?;

    if method.is_none() {
        anyhow::bail!("Method {} not found from {}", fullname, class.name()?);
    }
    let method = method.unwrap();

    tracing::trace!("get_java_method result {:#x}", method.ptr_raw);

    Ok(method.ptr_raw)
}

async fn java_jump_1(core: &mut ArmCore, _: &mut Jvm, arg1: u32, address: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_jump_1({:#x}, {:#x})", arg1, address);

    anyhow::ensure!(address != 0, "jump native address is null");

    Ok(core.run_function::<u32>(address, &[arg1]).await?)
}

async fn register_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> RuntimeResult<()> {
    tracing::trace!("register_class({:#x})", ptr_class);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    if jvm.has_class(&class.name()?).await {
        return Ok(());
    }

    jvm.register_class(Box::new(class), Some(KtfJvmSupport::class_loader(core)?)).await?;

    Ok(())
}

async fn register_java_string(core: &mut ArmCore, jvm: &mut Jvm, offset: u32, length: u32) -> RuntimeResult<u32> {
    tracing::trace!("register_java_string({:#x}, {:#x})", offset, length);

    let mut cursor = offset;
    let length = if length == 0xffff_ffff {
        let length: u16 = read_generic(core, offset)?;
        cursor += 2;
        length
    } else {
        length as _
    };

    let mut bytes = vec![0u8; (length * 2) as _];
    core.read_bytes(cursor, (length * 2) as _, &mut bytes)?;
    let bytes_u16 = bytes.chunks(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect::<Vec<_>>();

    let rust_string = String::from_utf16(&bytes_u16)?;

    let instance = JavaLangString::from_rust_string(jvm, &rust_string).await?;

    Ok(KtfJvmSupport::class_instance_raw(&instance) as _)
}

async fn get_static_field(core: &mut ArmCore, _jvm: &mut Jvm, ptr_class: u32, field_name: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub get_static_field({:#x}, {:#x})", ptr_class, field_name);

    let field_name = KtfJvmSupport::read_name(core, field_name)?;

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let field = class.field(&field_name.name, &field_name.descriptor, true)?.unwrap();

    Ok(field.ptr_raw)
}

async fn jb_unk4(_: &mut ArmCore, _: &mut Jvm, a0: u32, a1: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn jb_unk5(_: &mut ArmCore, _: &mut Jvm, a0: u32, a1: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn jb_unk7(_: &mut ArmCore, _: &mut Jvm, a0: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub jb_unk7({:#x})", a0);

    Ok(0)
}

async fn jb_unk8(_: &mut ArmCore, _: &mut Jvm, a0: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub jb_unk8({:#x})", a0);

    Ok(0)
}

async fn call_native(core: &mut ArmCore, _: &mut Jvm, address: u32, ptr_data: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_jump_native({:#x}, {:#x})", address, ptr_data);

    anyhow::ensure!(address != 0, "jump native address is null");

    let result = core.run_function::<u32>(address, &[ptr_data]).await?;

    write_generic(core, ptr_data, result)?;
    write_generic(core, ptr_data + 4, 0u32)?;

    Ok(ptr_data)
}

async fn java_jump_2(core: &mut ArmCore, _: &mut Jvm, arg1: u32, arg2: u32, address: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_jump_2({:#x}, {:#x}, {:#x})", arg1, arg2, address);

    anyhow::ensure!(address != 0, "jump native address is null");

    Ok(core.run_function::<u32>(address, &[arg1, arg2]).await?)
}

async fn java_jump_3(core: &mut ArmCore, _: &mut Jvm, arg1: u32, arg2: u32, arg3: u32, address: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_jump_3({:#x}, {:#x}, {:#x}, {:#x})", arg1, arg2, arg3, address);

    anyhow::ensure!(address != 0, "jump native address is null");

    Ok(core.run_function::<u32>(address, &[arg1, arg2, arg3]).await?)
}

pub async fn java_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_new({:#x})", ptr_class);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let class_name = class.name()?;

    let instance = jvm.instantiate_class(&class_name).await?;
    let raw = KtfJvmSupport::class_instance_raw(&instance);

    Ok(raw)
}

pub async fn java_array_new(core: &mut ArmCore, jvm: &mut Jvm, element_type: u32, count: u32) -> RuntimeResult<u32> {
    tracing::trace!("java_array_new({:#x}, {:#x})", element_type, count);

    let element_type_name = if element_type > 0x100 {
        // HACK: we don't have element type class
        let class = KtfJvmSupport::class_from_raw(core, element_type);
        class.name()?
    } else {
        (element_type as u8 as char).to_string()
    };

    let instance = jvm.instantiate_array(&element_type_name, count as _).await?;
    let raw = KtfJvmSupport::class_instance_raw(&instance);

    Ok(raw)
}

pub async fn java_check_cast(_: &mut ArmCore, _: &mut Jvm, ptr_class: u32, ptr_instance: u32) -> RuntimeResult<u32> {
    tracing::warn!("stub java_check_cast({:#x}, {:#x})", ptr_class, ptr_instance);

    Ok(1)
}
