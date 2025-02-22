use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::size_of;
use wie_jvm_support::JvmSupport;

use jvm::{Jvm, runtime::JavaLangString};

use bytemuck::{Pod, Zeroable};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::{ByteRead, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::runtime::java::jvm_support::{JavaClassDefinition, JavaMethod, JavaMethodResult, KtfJvmSupport, KtfJvmWord};

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

pub fn get_wipi_jb_interface(core: &mut ArmCore, jvm: &Jvm) -> Result<u32> {
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

pub async fn java_class_load(core: &mut ArmCore, jvm: &mut Jvm, ptr_target: u32, ptr_name: u32) -> Result<u32> {
    tracing::trace!("load_java_class({:#x}, {:#x})", ptr_target, ptr_name);

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?).unwrap();
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

pub async fn java_throw(core: &mut ArmCore, jvm: &mut Jvm, ptr_error: KtfJvmWord, a1: u32) -> Result<JavaMethodResult> {
    tracing::warn!("java_throw({:#x}, {})", ptr_error, a1);

    let error = String::from_utf8(read_null_terminated_string_bytes(core, ptr_error)?).unwrap();

    let exception = jvm.new_class(&error, "()V", ()).await.unwrap();

    JavaMethod::handle_exception(core, jvm, exception).await
}

async fn get_java_method(core: &mut ArmCore, _jvm: &mut Jvm, ptr_class: u32, ptr_fullname: u32) -> Result<u32> {
    let fullname = KtfJvmSupport::read_name(core, ptr_fullname)?;

    tracing::debug!("get_java_method({:#x}, {})", ptr_class, fullname);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let method = find_java_method(&class, &fullname.name, &fullname.descriptor).await?;

    if method.is_none() {
        return Err(WieError::FatalError(format!("Method {} not found from {}", fullname, class.name()?)));
    }
    let method = method.unwrap();

    tracing::trace!("get_java_method result {:#x}", method.ptr_raw);

    Ok(method.ptr_raw)
}

#[async_recursion::async_recursion]
async fn find_java_method(class: &JavaClassDefinition, name: &str, descriptor: &str) -> Result<Option<JavaMethod>> {
    let method = class.method(name, descriptor, false)?;
    let method = if method.is_none() {
        class.method(name, descriptor, true)?
    } else {
        method
    }; // TODO it's not good pattern...

    if method.is_none() {
        if let Some(x) = class.parent_class()? {
            return find_java_method(&x, name, descriptor).await;
        }
    }

    Ok(method)
}

async fn java_jump_1(core: &mut ArmCore, _: &mut Jvm, arg1: u32, address: u32) -> Result<u32> {
    tracing::trace!("java_jump_1({:#x}, {:#x})", arg1, address);

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    core.run_function::<u32>(address, &[arg1]).await
}

async fn register_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<()> {
    tracing::trace!("register_class({:#x})", ptr_class);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    if jvm.has_class(&class.name()?) {
        return Ok(());
    }

    let result = jvm.register_class(Box::new(class), Some(KtfJvmSupport::class_loader(core)?)).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    Ok(())
}

async fn register_java_string(core: &mut ArmCore, jvm: &mut Jvm, offset: u32, length: u32) -> Result<u32> {
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
    core.read_bytes(cursor, &mut bytes)?;
    let bytes_u16 = bytes.chunks(2).map(|x| u16::from_le_bytes([x[0], x[1]])).collect::<Vec<_>>();

    let rust_string = String::from_utf16(&bytes_u16).unwrap();

    let instance = JavaLangString::from_rust_string(jvm, &rust_string).await.unwrap();

    Ok(KtfJvmSupport::class_instance_raw(&instance) as _)
}

async fn get_static_field(core: &mut ArmCore, _jvm: &mut Jvm, ptr_class: u32, field_name: u32) -> Result<u32> {
    tracing::warn!("stub get_static_field({:#x}, {:#x})", ptr_class, field_name);

    let field_name = KtfJvmSupport::read_name(core, field_name)?;

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let field = class.field(&field_name.name, &field_name.descriptor, true)?;

    if let Some(x) = field {
        Ok(x.ptr_raw)
    } else {
        Err(WieError::FatalError(format!("Field {} not found from {}", field_name, class.name()?)))
    }
}

async fn jb_unk4(_: &mut ArmCore, _: &mut Jvm, a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn jb_unk5(_: &mut ArmCore, _: &mut Jvm, a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

async fn jb_unk7(_: &mut ArmCore, _: &mut Jvm, a0: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk7({:#x})", a0);

    Ok(0)
}

async fn jb_unk8(_: &mut ArmCore, _: &mut Jvm, a0: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk8({:#x})", a0);

    Ok(0)
}

async fn call_native(core: &mut ArmCore, _: &mut Jvm, address: u32, ptr_data: u32) -> Result<u32> {
    tracing::trace!("java_jump_native({:#x}, {:#x})", address, ptr_data);

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    // TODO correctly figure out parameter
    let result = core.run_function::<u32>(address, &[ptr_data, ptr_data]).await?;

    write_generic(core, ptr_data, result)?;
    write_generic(core, ptr_data + 4, 0u32)?;

    Ok(ptr_data)
}

async fn java_jump_2(core: &mut ArmCore, _: &mut Jvm, arg1: u32, arg2: u32, address: u32) -> Result<u32> {
    tracing::trace!("java_jump_2({:#x}, {:#x}, {:#x})", arg1, arg2, address);

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    core.run_function::<u32>(address, &[arg1, arg2]).await
}

async fn java_jump_3(core: &mut ArmCore, _: &mut Jvm, arg1: u32, arg2: u32, arg3: u32, address: u32) -> Result<u32> {
    tracing::trace!("java_jump_3({:#x}, {:#x}, {:#x}, {:#x})", arg1, arg2, arg3, address);

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    core.run_function::<u32>(address, &[arg1, arg2, arg3]).await
}

pub async fn java_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<u32> {
    tracing::trace!("java_new({:#x})", ptr_class);

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let class_name = class.name()?;

    let result = jvm.instantiate_class(&class_name).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    let raw = KtfJvmSupport::class_instance_raw(&result.unwrap());

    Ok(raw)
}

pub async fn java_array_new(core: &mut ArmCore, jvm: &mut Jvm, element_type: u32, count: u32) -> Result<u32> {
    tracing::trace!("java_array_new({:#x}, {:#x})", element_type, count);

    let element_type_name = if element_type > 0x100 {
        // HACK: we don't have element type class
        let class = KtfJvmSupport::class_from_raw(core, element_type);
        class.name()?[1..].into()
    } else {
        (element_type as u8 as char).to_string()
    };
    let result = jvm.instantiate_array(&element_type_name, count as _).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    let raw = KtfJvmSupport::class_instance_raw(&result.unwrap());

    Ok(raw)
}

pub async fn java_array_store_check_object_type(_: &mut ArmCore, _: &mut Jvm, vtable: u32, value: u32) -> Result<u32> {
    tracing::warn!("stub java_array_store_check_object_type({:#x}, {:#x})", vtable, value);

    Ok(1)
}
