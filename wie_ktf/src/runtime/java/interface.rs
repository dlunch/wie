use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::size_of;

use java_runtime::classes::java::util::Vector;
use jvm::{ClassInstanceRef, Jvm, runtime::JavaLangString};
use wipi_types::ktf::java::WIPIJBInterface;

use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{ByteRead, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::runtime::java::jvm_support::{
    JavaClassDefinition, JavaClassInstance, JavaMethod, JavaMethodResult, JavaVtable, KtfJvmSupport, KtfJvmWord,
};
use crate::runtime::{SVC_CATEGORY_JAVA_INTERFACE, svc_ids::JavaSvcId};

pub(crate) fn register_java_interface_svc_handler(core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA_INTERFACE, handle_java_interface_svc, jvm)
}

async fn handle_java_interface_svc(core: &mut ArmCore, jvm: &mut Jvm, id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    match JavaSvcId::try_from(id)? {
        JavaSvcId::JavaJump1 => EmulatedFunction::call(&java_jump_1, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JavaJump2 => EmulatedFunction::call(&java_jump_2, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JavaJump3 => EmulatedFunction::call(&java_jump_3, core, &mut ()).await?.write(core, lr),
        JavaSvcId::GetJavaMethod => EmulatedFunction::call(&get_java_method, core, &mut ()).await?.write(core, lr),
        JavaSvcId::GetField => EmulatedFunction::call(&get_field, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JbUnk4 => EmulatedFunction::call(&jb_unk4, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JbUnk5 => EmulatedFunction::call(&jb_unk5, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JbUnk7 => EmulatedFunction::call(&jb_unk7, core, &mut ()).await?.write(core, lr),
        JavaSvcId::JbUnk8 => EmulatedFunction::call(&jb_unk8, core, &mut ()).await?.write(core, lr),
        JavaSvcId::RegisterClass => EmulatedFunction::call(&register_class, core, jvm).await?.write(core, lr),
        JavaSvcId::RegisterJavaString => EmulatedFunction::call(&register_java_string, core, jvm).await?.write(core, lr),
        JavaSvcId::CallNative => EmulatedFunction::call(&call_native, core, &mut ()).await?.write(core, lr),
    }
}

pub fn get_wipi_jb_interface(core: &mut ArmCore) -> Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_java_jump_1: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump1 as u32)?,
        fn_java_jump_2: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump2 as u32)?,
        fn_java_jump_3: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump3 as u32)?,
        fn_get_java_method: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::GetJavaMethod as u32)?,
        fn_get_field: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::GetField as u32)?,
        fn_unk4: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JbUnk4 as u32)?,
        fn_unk5: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JbUnk5 as u32)?,
        fn_unk7: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JbUnk7 as u32)?,
        fn_unk8: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JbUnk8 as u32)?,
        fn_register_class: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::RegisterClass as u32)?,
        fn_register_java_string: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::RegisterJavaString as u32)?,
        fn_call_native: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::CallNative as u32)?,
    };

    let address = Allocator::alloc(core, size_of::<WIPIJBInterface>() as u32)?;
    write_generic(core, address, interface)?;

    Ok(address)
}

pub async fn java_class_load(core: &mut ArmCore, jvm: &mut Jvm, ptr_target: u32, ptr_name: u32) -> Result<u32> {
    tracing::trace!("load_java_class({ptr_target:#x}, {ptr_name:#x})");

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?).unwrap();
    let class = jvm.resolve_class(&name).await;

    if let Ok(x) = class {
        let raw = KtfJvmSupport::class_definition_raw(&*x.definition)?;
        write_generic(core, ptr_target, raw)?;

        Ok(0)
    } else {
        tracing::error!("load_java_class({name}) failed");

        Ok(1)
    }
}

pub async fn java_throw(core: &mut ArmCore, jvm: &mut Jvm, ptr_error: KtfJvmWord, a1: u32) -> Result<JavaMethodResult> {
    tracing::warn!("java_throw({ptr_error:#x}, {a1})");

    let error = String::from_utf8(read_null_terminated_string_bytes(core, ptr_error)?).unwrap();

    let exception = jvm.new_class(&error, "()V", ()).await.unwrap();

    JavaMethod::handle_exception(core, jvm, exception).await
}

fn map_jump_result(result: core::result::Result<u32, WieError>) -> Result<JavaMethodResult> {
    match result {
        Ok(result) => Ok(JavaMethodResult::new(vec![result], None)),
        Err(WieError::JavaExceptionUnwind {
            context_base,
            target,
            next_pc,
        }) => Ok(JavaMethodResult::new(vec![context_base, target], Some(next_pc))),
        Err(err) => Err(err),
    }
}

async fn get_java_method(core: &mut ArmCore, _: &mut (), ptr_class: u32, ptr_fullname: u32) -> Result<u32> {
    let fullname = KtfJvmSupport::read_name(core, ptr_fullname)?;

    tracing::debug!("get_java_method({ptr_class:#x}, {fullname})");

    // ptr_class might be vtable
    let first_item: u32 = read_generic(core, ptr_class)?;
    let method = if first_item != ptr_class + 4 {
        // ptr_class is pointer to vtable
        let vtable = JavaVtable::from_raw(core, first_item);
        let method = vtable.find_method(&fullname.name, &fullname.descriptor)?;

        if method.is_none() {
            return Err(WieError::FatalError(format!("Method {fullname} not found from {ptr_class:#x}")));
        }
        method
    } else {
        let class = KtfJvmSupport::class_from_raw(core, ptr_class);
        let method = find_java_method(&class, &fullname.name, &fullname.descriptor).await?;

        if method.is_none() {
            return Err(WieError::FatalError(format!("Method {fullname} not found from {}", class.name()?)));
        }

        method
    };
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

    if method.is_none()
        && let Some(x) = class.parent_class()?
    {
        return find_java_method(&x, name, descriptor).await;
    }

    Ok(method)
}

async fn java_jump_1(core: &mut ArmCore, _: &mut (), arg1: u32, address: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_1({arg1:#x}, {address:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    map_jump_result(core.run_function::<u32>(address, &[arg1, 0, 0]).await)
}

async fn register_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<()> {
    tracing::trace!("register_class({ptr_class:#x})");

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    if jvm.has_class(&class.name()?) {
        return Ok(());
    }

    let ktf_class_loader = jvm
        .get_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;")
        .await
        .unwrap();

    let result = jvm.register_class(Box::new(class), Some(ktf_class_loader)).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    Ok(())
}

async fn register_java_string(core: &mut ArmCore, jvm: &mut Jvm, offset: u32, length: u32) -> Result<u32> {
    tracing::trace!("register_java_string({offset:#x}, {length:#x})");

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

    // Add to class loader to not to be garbage collected
    // TODO encapsulation?
    let ktf_class_loader = jvm
        .get_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;")
        .await
        .unwrap();
    let strings_field: ClassInstanceRef<Vector> = jvm.get_field(&ktf_class_loader, "nativeStrings", "Ljava/util/Vector;").await.unwrap();
    let _: bool = jvm
        .invoke_virtual(&strings_field, "add", "(Ljava/lang/Object;)Z", (instance.clone(),))
        .await
        .unwrap();

    Ok(KtfJvmSupport::class_instance_raw(&instance) as _)
}

async fn get_field(core: &mut ArmCore, _: &mut (), ptr_class: u32, field_name: u32) -> Result<u32> {
    tracing::debug!("get_field({ptr_class:#x}, {field_name:#x})");

    let field_name = KtfJvmSupport::read_name(core, field_name)?;

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let field = class.field(&field_name.name, &field_name.descriptor, true)?;
    let field = if field.is_none() {
        class.field(&field_name.name, &field_name.descriptor, false)?
    } else {
        field
    };

    if let Some(x) = field {
        Ok(x.ptr_raw)
    } else {
        Err(WieError::FatalError(format!("Field {field_name} not found from {}", class.name()?)))
    }
}

async fn jb_unk4(_: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk4({a0:#x}, {a1:#x})");

    Ok(0)
}

async fn jb_unk5(_: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk5({a0:#x}, {a1:#x})");

    Ok(0)
}

async fn jb_unk7(_: &mut ArmCore, _: &mut (), a0: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk7({a0:#x})");

    Ok(0)
}

async fn jb_unk8(_: &mut ArmCore, _: &mut (), a0: u32) -> Result<u32> {
    tracing::warn!("stub jb_unk8({a0:#x})");

    Ok(0)
}

async fn call_native(core: &mut ArmCore, _: &mut (), address: u32, ptr_data: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_native({address:#x}, {ptr_data:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    // TODO correctly figure out parameter
    let result = match core.run_function::<u32>(address, &[ptr_data, ptr_data]).await {
        Ok(result) => result,
        Err(WieError::JavaExceptionUnwind {
            context_base,
            target,
            next_pc,
        }) => return Ok(JavaMethodResult::new(vec![context_base, target], Some(next_pc))),
        Err(err) => return Err(err),
    };

    write_generic(core, ptr_data, result)?;
    write_generic(core, ptr_data + 4, 0u32)?;

    Ok(JavaMethodResult::new(vec![ptr_data], None))
}

async fn java_jump_2(core: &mut ArmCore, _: &mut (), arg1: u32, arg2: u32, address: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_2({arg1:#x}, {arg2:#x}, {address:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    map_jump_result(core.run_function::<u32>(address, &[arg1, arg2, 0]).await)
}

async fn java_jump_3(core: &mut ArmCore, _: &mut (), arg1: u32, arg2: u32, arg3: u32, address: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_3({arg1:#x}, {arg2:#x}, {arg3:#x}, {address:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    map_jump_result(core.run_function::<u32>(address, &[arg1, arg2, arg3]).await)
}

pub async fn java_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<u32> {
    tracing::trace!("java_new({ptr_class:#x})");

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
    tracing::trace!("java_array_new({element_type:#x}, {count:#x})");

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

pub async fn java_check_type(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32, ptr_instance: u32, unk: u32) -> Result<u32> {
    tracing::warn!("stub java_check_type({ptr_class:#x}, {ptr_instance:#x}, {unk:#x})");

    let instance = JavaClassInstance::from_raw(ptr_instance, core);

    // TODO is it correct?
    if instance.class()?.name()?.starts_with('[') || unk != 0 {
        return Ok(1);
    }

    let class = JavaClassDefinition::from_raw(ptr_class, core);
    let result = jvm.is_instance(&instance, &class.name()?);

    Ok(if result { 1 } else { 0 })
}
