use alloc::{boxed::Box, format, string::String, string::ToString, vec::Vec};
use core::mem::size_of;

use java_runtime::classes::java::util::Vector;
use jvm::{
    ClassDefinition, ClassInstance, ClassInstanceRef, JavaError, Jvm,
    runtime::{JavaLangClass, JavaLangClassLoader, JavaLangString},
};
use wipi_types::lgt::java::{LgtJavaClass as RawJavaClass, LgtJavaClassDescriptor as RawJavaClassDescriptor, LgtJavaClassLink as RawJavaClassLink};

use wie_core_arm::{ArmCore, EmulatedFunction, JumpTo, ResultWriter, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::runtime::{
    SVC_CATEGORY_JAVA_SYSTEM,
    java::{
        abi::{CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR},
        exception,
        jvm_support::LgtJvmSupport,
    },
    svc_ids::JavaSystemSvcId,
};

pub fn get_java_interface_method(core: &mut ArmCore, function_index: u32) -> Result<u32> {
    Ok(match function_index {
        0x03 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::InterfaceUnk0)?,
        0x06 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::DestroyRuntimeContext)?,
        0x07 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::CreateRuntimeContext)?,
        0x09 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::StringLiteral)?,
        0x0b => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::RegisterClass)?,
        0x0c => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::ResolveClass)?,
        0x0d => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::InitializeClass)?,
        0x0e => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::GetArrayType)?,
        0x0f => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Instantiate)?,
        0x10 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::InstantiateArray)?,
        0x11 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::InstantiateMultiArray)?,
        0x12 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::ExceptionMatchesClass)?,
        0x13 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::LinkPublicClass)?,
        0x14 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::LinkImportedClasses)?,
        0x1f => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::PushExceptionFrame)?,
        0x20 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::PopExceptionFrame)?,
        0x21 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::RethrowException)?,
        0x22 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::RaiseNullPointerException)?,
        0x23 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::RaiseArrayIndexException)?,
        0x25 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::RaiseArithmeticException)?,
        0x54 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Unk54)?,
        0x55 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Unk55)?,
        0x61 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::StoreReferenceArray)?,
        0x82 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::SetJarPath)?,
        0x83 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::StartApplication)?,
        0xe1 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::GetStringClass)?,
        0xe2 => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::GetStringArrayClass)?,
        0xfa => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::StoreReferenceArrayUnchecked)?,
        _ => return Err(WieError::FatalError(format!("Unknown lgt java import: {function_index:#x}"))),
    })
}

pub fn register_java_system_svc_handler(core: &mut ArmCore, jvm: &Jvm, ptr_jar_path: u32) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA_SYSTEM, handle_java_system_svc, &(jvm.clone(), ptr_jar_path))
}

async fn handle_java_system_svc(core: &mut ArmCore, (jvm, ptr_jar_path): &mut (Jvm, u32), id: SvcId) -> Result<JumpTo> {
    let (_, lr) = core.read_pc_lr()?;
    let result: Result<()> = async {
        match JavaSystemSvcId::try_from(id)? {
            JavaSystemSvcId::InterfaceUnk0 => EmulatedFunction::call(&java_unk0, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::DestroyRuntimeContext => EmulatedFunction::call(&java_destroy_runtime_context, core, &mut ())
                .await?
                .write(core, lr),
            JavaSystemSvcId::CreateRuntimeContext => EmulatedFunction::call(&java_create_runtime_context, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::LinkImportedClasses => EmulatedFunction::call(&java_link_imported_classes, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::SetJarPath => EmulatedFunction::call(&java_set_jar_path, core, ptr_jar_path).await?.write(core, lr),
            JavaSystemSvcId::StartApplication => EmulatedFunction::call(&java_start_application, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::RegisterClass => EmulatedFunction::call(&java_register_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::ResolveClass => EmulatedFunction::call(&java_resolve_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InitializeClass => EmulatedFunction::call(&java_initialize_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetArrayType => EmulatedFunction::call(&java_get_array_type, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::Instantiate => EmulatedFunction::call(&java_instantiate, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InstantiateArray => EmulatedFunction::call(&java_instantiate_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::InstantiateMultiArray => EmulatedFunction::call(&java_instantiate_multi_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::Unk54 => EmulatedFunction::call(&java_unk54, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Unk55 => EmulatedFunction::call(&java_unk55, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::StringLiteral => EmulatedFunction::call(&java_string_literal, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::PushExceptionFrame => EmulatedFunction::call(&java_push_exception_frame, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::PopExceptionFrame => EmulatedFunction::call(&java_pop_exception_frame, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::StoreReferenceArray => EmulatedFunction::call(&java_store_reference_array, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetStringClass => EmulatedFunction::call(&java_get_string_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::GetStringArrayClass => EmulatedFunction::call(&java_get_string_array_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::PendingException => EmulatedFunction::call(&java_pending_exception, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::StoreReferenceArrayUnchecked => EmulatedFunction::call(&java_store_reference_array_unchecked, core, &mut ())
                .await?
                .write(core, lr),
            JavaSystemSvcId::LinkPublicClass => EmulatedFunction::call(&java_link_public_class, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::ExceptionMatchesClass => {
                java_exception_matches_class(core, jvm, core.read_param(0)?, core.read_param(1)?, core.read_param(2)?)
                    .await?
                    .write(core, lr)
            }
            JavaSystemSvcId::RethrowException => EmulatedFunction::call(&java_rethrow_exception, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::RaiseNullPointerException => EmulatedFunction::call(&java_raise_null_pointer_exception, core, jvm)
                .await?
                .write(core, lr),
            JavaSystemSvcId::RaiseArrayIndexException => EmulatedFunction::call(&java_raise_array_index_exception, core, jvm)
                .await?
                .write(core, lr),
            JavaSystemSvcId::RaiseArithmeticException => EmulatedFunction::call(&java_raise_arithmetic_exception, core, jvm).await?.write(core, lr),
            JavaSystemSvcId::Unk1 => EmulatedFunction::call(&java_unk1, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Unk2 => EmulatedFunction::call(&java_unk2, core, &mut ()).await?.write(core, lr),
            JavaSystemSvcId::Unk3 => EmulatedFunction::call(&java_unk3, core, &mut ()).await?.write(core, lr),
        }
    }
    .await;

    match result {
        Ok(()) => Ok(JumpTo(lr)),
        Err(WieError::JavaException(ptr_exception)) => match exception::unwind(core, ptr_exception)? {
            Some(resume_address) => Ok(JumpTo(resume_address)),
            None => Err(WieError::JavaException(ptr_exception)),
        },
        Err(error) => Err(error),
    }
}

async fn java_unk1(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk1({a0:#x}, {a1:#x}, {a2:#x})");
    Ok(())
}

async fn java_unk2(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk2({a0:#x}, {a1:#x}, {a2:#x})");
    Ok(())
}

async fn java_unk3(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk3({a0:#x}, {a1:#x}, {a2:#x})");
    Ok(())
}

async fn java_unk54(_core: &mut ArmCore, _: &mut ()) -> Result<()> {
    Ok(())
}

async fn java_unk55(_core: &mut ArmCore, _: &mut ()) -> Result<()> {
    Ok(())
}

async fn java_string_literal(core: &mut ArmCore, jvm: &mut Jvm, _runtime_context: u32, data: u32, length: u32, cache: u32) -> Result<u32> {
    let cached: u32 = read_generic(core, cache)?;
    if cached != 0 {
        return Ok(cached);
    }

    let characters = (0..length)
        .map(|index| read_generic(core, data + index * size_of::<u16>() as u32))
        .collect::<Result<Vec<u16>>>()?;
    let value = String::from_utf16(&characters).map_err(|error| WieError::FatalError(format!("Invalid LGT UTF-16 literal: {error}")))?;
    let value = JavaLangString::from_rust_string(jvm, &value)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let class_loader: Box<dyn ClassInstance> = jvm
        .get_static_field("net/wie/LgtClassLoader", "instance", "Lnet/wie/LgtClassLoader;")
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let native_strings: ClassInstanceRef<Vector> = jvm
        .get_field(&class_loader, "nativeStrings", "Ljava/util/Vector;")
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let _: bool = jvm
        .invoke_virtual(&native_strings, "add", "(Ljava/lang/Object;)Z", (value.clone(),))
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let value = LgtJvmSupport::class_instance_raw(&*value);
    write_generic(core, cache, value)?;
    Ok(value)
}

async fn java_push_exception_frame(core: &mut ArmCore, _: &mut ()) -> Result<()> {
    exception::push(core)
}

async fn java_pop_exception_frame(core: &mut ArmCore, _: &mut ()) -> Result<()> {
    exception::pop(core)
}

async fn java_pending_exception(core: &mut ArmCore, _: &mut ()) -> Result<u32> {
    exception::pending(core)
}

async fn java_exception_matches_class(core: &mut ArmCore, jvm: &Jvm, _ptr_exception_type: u32, ptr_class_name: u32, _ptr_fields: u32) -> Result<u32> {
    let class_name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_class_name)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT exception class name: {error}")))?;
    let ptr_exception = exception::pending(core)?;
    let exception = LgtJvmSupport::class_instance_from_raw(core, ptr_exception);

    Ok(u32::from(jvm.is_instance(&*exception, &class_name)))
}

async fn java_rethrow_exception(core: &mut ArmCore, _: &mut (), ptr_exception: u32) -> Result<()> {
    exception::pop(core)?;
    Err(WieError::JavaException(ptr_exception))
}

async fn java_raise_null_pointer_exception(_core: &mut ArmCore, jvm: &mut Jvm) -> Result<()> {
    let JavaError::JavaException(exception) = jvm.exception("java/lang/NullPointerException", "").await;
    Err(WieError::JavaException(LgtJvmSupport::class_instance_raw(&*exception)))
}

async fn java_raise_array_index_exception(_core: &mut ArmCore, jvm: &mut Jvm, index: u32) -> Result<()> {
    let JavaError::JavaException(exception) = jvm.exception("java/lang/ArrayIndexOutOfBoundsException", &index.to_string()).await;
    Err(WieError::JavaException(LgtJvmSupport::class_instance_raw(&*exception)))
}

async fn java_raise_arithmetic_exception(_core: &mut ArmCore, jvm: &mut Jvm) -> Result<()> {
    let JavaError::JavaException(exception) = jvm.exception("java/lang/ArithmeticException", "/ by zero").await;
    Err(WieError::JavaException(LgtJvmSupport::class_instance_raw(&*exception)))
}

async fn java_store_reference_array_unchecked(core: &mut ArmCore, _: &mut (), ptr_array: u32, index: u32, ptr_value: u32) -> Result<()> {
    let ptr_fields: u32 = read_generic(core, ptr_array + 2 * size_of::<u32>() as u32)?;
    write_generic(core, ptr_fields + (index + 1) * size_of::<u32>() as u32, ptr_value)
}

async fn java_store_reference_array(core: &mut ArmCore, jvm: &mut Jvm, ptr_array: u32, index: u32, ptr_value: u32) -> Result<()> {
    let mut array = LgtJvmSupport::class_instance_from_raw(core, ptr_array);
    let value = (ptr_value != 0).then(|| LgtJvmSupport::class_instance_from_raw(core, ptr_value));
    if let Some(value) = &value
        && !jvm.array_store_allowed(&*array, &**value)
    {
        let JavaError::JavaException(exception) = jvm.exception("java/lang/ArrayStoreException", "incompatible array element").await;
        return Err(WieError::JavaException(LgtJvmSupport::class_instance_raw(&*exception)));
    }
    jvm.store_array(&mut array, index as usize, [jvm::JavaValue::Object(value)])
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))
}

async fn java_get_string_class(_core: &mut ArmCore, jvm: &mut Jvm) -> Result<u32> {
    let class = jvm
        .resolve_class("java/lang/String")
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    Ok(LgtJvmSupport::class_instance_raw(&*class.java_class()))
}

async fn java_get_string_array_class(_core: &mut ArmCore, jvm: &mut Jvm) -> Result<u32> {
    let class = jvm
        .resolve_class("[Ljava/lang/String;")
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    Ok(LgtJvmSupport::class_instance_raw(&*class.java_class()))
}

async fn java_unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk0({a0:#x}, {a1:#x}, {a2:#x})");

    Ok(())
}

async fn java_create_runtime_context(_core: &mut ArmCore, jvm: &mut Jvm, generated_classes: u32, _runtime_metadata: u32) -> Result<u32> {
    tracing::debug!("java_create_runtime_context({generated_classes:#x})");

    let parent: Box<dyn ClassInstance> = jvm
        .invoke_static("java/lang/ClassLoader", "getSystemClassLoader", "()Ljava/lang/ClassLoader;", ())
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let _: Box<dyn ClassInstance> = jvm
        .new_class(
            "net/wie/LgtClassLoader",
            "(Ljava/lang/ClassLoader;I)V",
            (parent, generated_classes as i32),
        )
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;

    Ok(generated_classes)
}

async fn java_register_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<()> {
    let class = LgtJvmSupport::class_from_raw(core, ptr_class);
    if class.descriptor()?.link_state == 3 {
        return Ok(());
    }

    let loader: Box<dyn ClassInstance> = jvm
        .get_static_field("net/wie/LgtClassLoader", "instance", "Lnet/wie/LgtClassLoader;")
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    LgtJvmSupport::register_generated_class(core, jvm, ptr_class, loader).await?;

    Ok(())
}

async fn java_resolve_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32, _runtime_context: u32) -> Result<u32> {
    java_register_class(core, jvm, ptr_class).await?;

    let name = ClassDefinition::name(&LgtJvmSupport::class_from_raw(core, ptr_class));
    let class = jvm
        .get_class(&name)
        .ok_or_else(|| WieError::FatalError(format!("LGT generated class not resolved: {name}")))?;

    Ok(LgtJvmSupport::class_instance_raw(&*class.java_class()))
}

async fn java_initialize_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class_object: u32, callback: u32) -> Result<()> {
    let mut class_object = LgtJvmSupport::class_instance_from_raw(core, ptr_class_object);
    let ready: i32 = jvm
        .get_field(&class_object, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    if ready == 5 {
        return Ok(());
    }

    if callback != 0 {
        let _: () = core.run_function(callback, &[]).await?;
    }
    jvm.put_field(&mut class_object, CLASS_INITIALIZATION_STATE_FIELD, WORD_FIELD_DESCRIPTOR, 5i32)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))
}

async fn java_get_array_type(core: &mut ArmCore, jvm: &mut Jvm, rank: u32, ptr_component_name: u32, primitive_type: u32) -> Result<u32> {
    let component = if ptr_component_name != 0 {
        let component = String::from_utf8(read_null_terminated_string_bytes(core, ptr_component_name)?)
            .map_err(|error| WieError::FatalError(format!("Invalid LGT array component name: {error}")))?;
        if component.starts_with('[') {
            component
        } else {
            format!("L{component};")
        }
    } else {
        String::from(match primitive_type {
            4 => "Z",
            5 => "C",
            6 => "F",
            7 => "D",
            8 => "B",
            9 => "S",
            10 => "I",
            11 => "J",
            _ => return Err(WieError::FatalError(format!("Unknown LGT primitive array type {primitive_type}"))),
        })
    };
    let name = format!("{}{component}", "[".repeat(rank as usize));

    let class = jvm
        .resolve_class(&name)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;

    Ok(LgtJvmSupport::class_instance_raw(&*class.java_class()))
}

async fn java_instantiate(core: &mut ArmCore, jvm: &mut Jvm, ptr_class_object: u32) -> Result<u32> {
    let class_object = LgtJvmSupport::class_instance_from_raw(core, ptr_class_object);
    let definition = JavaLangClass::to_rust_class(jvm, &class_object)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let instance = definition
        .instantiate(jvm)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let ptr_instance = LgtJvmSupport::class_instance_raw(&*instance);

    let mut initializer_callbacks = Vec::new();
    let mut current = Some(definition);
    while let Some(definition) = current {
        let ptr_raw = LgtJvmSupport::class_definition_raw(&*definition);
        let raw: RawJavaClass = read_generic(core, ptr_raw)?;
        let descriptor: RawJavaClassDescriptor = read_generic(core, raw.ptr_descriptor)?;
        if descriptor.ptr_vtable != 0 {
            let callback: u32 = read_generic(core, descriptor.ptr_vtable + size_of::<u32>() as u32)?;
            if callback != 0 {
                initializer_callbacks.push(callback);
            }
        }
        current = if let Some(parent_name) = definition.super_class_name() {
            Some(
                jvm.resolve_class(&parent_name)
                    .await
                    .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?
                    .definition,
            )
        } else {
            None
        };
    }
    for callback in initializer_callbacks.into_iter().rev() {
        let _: () = core.run_function(callback, &[ptr_instance]).await?;
    }

    Ok(ptr_instance)
}

async fn java_instantiate_array(core: &mut ArmCore, jvm: &mut Jvm, ptr_class_object: u32, length: u32) -> Result<u32> {
    let class_object = LgtJvmSupport::class_instance_from_raw(core, ptr_class_object);
    let name = JavaLangClass::name(jvm, &class_object)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let element_type = name
        .strip_prefix('[')
        .ok_or_else(|| WieError::FatalError(format!("Not an LGT array class: {name}")))?;
    let array = jvm
        .instantiate_array(element_type, length as usize)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;

    Ok(LgtJvmSupport::class_instance_raw(&*array))
}

async fn java_instantiate_multi_array(core: &mut ArmCore, jvm: &mut Jvm, ptr_class_object: u32, ptr_dimensions: u32, rank: u32) -> Result<u32> {
    let class_object = LgtJvmSupport::class_instance_from_raw(core, ptr_class_object);
    let name = JavaLangClass::name(jvm, &class_object)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let dimensions = (0..rank)
        .map(|index| read_generic::<u32, _>(core, ptr_dimensions + index * size_of::<u32>() as u32).map(|value| value as usize))
        .collect::<Result<Vec<_>>>()?;

    let deepest_level = dimensions.len() - 1;
    let deepest_count = dimensions[..deepest_level].iter().product();
    let mut arrays = Vec::with_capacity(deepest_count);
    for _ in 0..deepest_count {
        arrays.push(
            jvm.instantiate_array(&name[deepest_level + 1..], dimensions[deepest_level])
                .await
                .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?,
        );
    }

    for level in (0..deepest_level).rev() {
        let parent_count = dimensions[..level].iter().product();
        let mut parents = Vec::with_capacity(parent_count);
        let mut children = arrays.into_iter();
        for _ in 0..parent_count {
            let mut parent = jvm
                .instantiate_array(&name[level + 1..], dimensions[level])
                .await
                .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
            jvm.store_array(&mut parent, 0, children.by_ref().take(dimensions[level]).collect::<Vec<_>>())
                .await
                .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
            parents.push(parent);
        }
        arrays = parents;
    }

    Ok(LgtJvmSupport::class_instance_raw(&*arrays.remove(0)))
}

fn read_member_name_and_descriptor(core: &ArmCore, table: u32, index: u16) -> Result<(String, String)> {
    let ptr_name: u32 = read_generic(core, table + index as u32 * 2 * size_of::<u32>() as u32)?;
    let ptr_descriptor: u32 = read_generic(core, table + (index as u32 * 2 + 1) * size_of::<u32>() as u32)?;
    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT member name: {error}")))?;
    let descriptor = String::from_utf8(read_null_terminated_string_bytes(core, ptr_descriptor)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT member descriptor: {error}")))?;
    Ok((name, descriptor))
}

#[allow(clippy::too_many_arguments)]
async fn link_class_members(
    core: &mut ArmCore,
    jvm: &Jvm,
    class_name: &str,
    link: RawJavaClassLink,
    instance_field_imports: u32,
    static_field_imports: u32,
    virtual_method_imports: u32,
    interface_method_imports: u32,
    non_virtual_method_imports: u32,
    instance_field_word_indices: u32,
    static_field_word_indices: u32,
    virtual_method_indices: u32,
    interface_method_indices: u32,
    non_virtual_method_targets: u32,
) -> Result<()> {
    for index in link.instance_field_offset..link.instance_field_offset + link.instance_field_count {
        let (name, descriptor) = read_member_name_and_descriptor(core, instance_field_imports, index)?;
        let word_index = LgtJvmSupport::field_word_index(jvm, class_name, &name, &descriptor, false)?;
        write_generic(core, instance_field_word_indices + index as u32 * size_of::<u16>() as u32, word_index)?;
    }

    for index in link.static_field_offset..link.static_field_offset + link.static_field_count {
        let (name, descriptor) = read_member_name_and_descriptor(core, static_field_imports, index)?;
        let word_index = LgtJvmSupport::field_word_index(jvm, class_name, &name, &descriptor, true)?;
        write_generic(core, static_field_word_indices + index as u32 * size_of::<u16>() as u32, word_index)?;
    }

    for index in link.virtual_method_offset..link.virtual_method_offset + link.virtual_method_count {
        let (name, descriptor) = read_member_name_and_descriptor(core, virtual_method_imports, index)?;
        let method_index = LgtJvmSupport::virtual_method_index(jvm, class_name, &name, &descriptor).await?;
        write_generic(core, virtual_method_indices + index as u32 * size_of::<u16>() as u32, method_index)?;
    }

    if link.interface_method_count != 0 {
        return Err(WieError::FatalError(format!(
            "Interface method linking is not implemented for {class_name}: table {interface_method_imports:#x}, output {interface_method_indices:#x}, range {}..{}",
            link.interface_method_offset,
            link.interface_method_offset + link.interface_method_count
        )));
    }

    let (initialized_class_getter, class_getter) = LgtJvmSupport::class_getter_targets(jvm, class_name)?;
    for local_index in 0..link.non_virtual_method_count {
        let index = link.non_virtual_method_offset + local_index;
        let target = match local_index {
            0 => initialized_class_getter,
            1 => class_getter,
            _ => {
                let (name, descriptor) = read_member_name_and_descriptor(core, non_virtual_method_imports, index)?;
                LgtJvmSupport::non_virtual_method_target(jvm, class_name, &name, &descriptor)?
            }
        };
        write_generic(core, non_virtual_method_targets + index as u32 * size_of::<u32>() as u32, target)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn java_link_public_class(
    core: &mut ArmCore,
    jvm: &mut Jvm,
    ptr_link: u32,
    ptr_class_object: u32,
    instance_field_imports: u32,
    static_field_imports: u32,
    virtual_method_imports: u32,
    interface_method_imports: u32,
    non_virtual_method_imports: u32,
    instance_field_word_indices: u32,
    static_field_word_indices: u32,
    virtual_method_indices: u32,
    interface_method_indices: u32,
) -> Result<()> {
    let non_virtual_method_targets = core.read_param(11)?;
    let link: RawJavaClassLink = read_generic(core, ptr_link)?;
    let class_object = LgtJvmSupport::class_instance_from_raw(core, ptr_class_object);
    let class_name = JavaLangClass::name(jvm, &class_object)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    link_class_members(
        core,
        jvm,
        &class_name,
        link,
        instance_field_imports,
        static_field_imports,
        virtual_method_imports,
        interface_method_imports,
        non_virtual_method_imports,
        instance_field_word_indices,
        static_field_word_indices,
        virtual_method_indices,
        interface_method_indices,
        non_virtual_method_targets,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn java_link_imported_classes(
    core: &mut ArmCore,
    jvm: &mut Jvm,
    imported_classes: u32,
    instance_field_imports: u32,
    static_field_imports: u32,
    virtual_method_imports: u32,
    interface_method_imports: u32,
    non_virtual_method_imports: u32,
    instance_field_word_indices: u32,
    static_field_word_indices: u32,
    virtual_method_indices: u32,
    interface_method_indices: u32,
    non_virtual_method_targets: u32,
) -> Result<()> {
    let class_count: u32 = read_generic(core, imported_classes)?;
    for index in 0..class_count {
        let link: RawJavaClassLink = read_generic(
            core,
            imported_classes + size_of::<u32>() as u32 + index * size_of::<RawJavaClassLink>() as u32,
        )?;
        let class_name = String::from_utf8(read_null_terminated_string_bytes(core, link.ptr_name)?)
            .map_err(|error| WieError::FatalError(format!("Invalid LGT imported class name: {error}")))?;
        tracing::debug!(
            "Linking imported class {class_name}: instance {}+{}, static {}+{}, virtual {}+{}, interface {}+{}, direct {}+{}",
            link.instance_field_offset,
            link.instance_field_count,
            link.static_field_offset,
            link.static_field_count,
            link.virtual_method_offset,
            link.virtual_method_count,
            link.interface_method_offset,
            link.interface_method_count,
            link.non_virtual_method_offset,
            link.non_virtual_method_count
        );
        for local_index in 2..link.non_virtual_method_count {
            let member_index = link.non_virtual_method_offset + local_index;
            let (name, descriptor) = read_member_name_and_descriptor(core, non_virtual_method_imports, member_index)?;
            tracing::debug!("Imported direct method {class_name}.{name}{descriptor}");
        }
        jvm.resolve_class(&class_name)
            .await
            .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
        link_class_members(
            core,
            jvm,
            &class_name,
            link,
            instance_field_imports,
            static_field_imports,
            virtual_method_imports,
            interface_method_imports,
            non_virtual_method_imports,
            instance_field_word_indices,
            static_field_word_indices,
            virtual_method_indices,
            interface_method_indices,
            non_virtual_method_targets,
        )
        .await?;
    }

    Ok(())
}

async fn java_set_jar_path(core: &mut ArmCore, ptr_jar_path: &mut u32, jar_path: u32) -> Result<()> {
    write_generic(core, *ptr_jar_path, jar_path)
}

async fn java_start_application(
    core: &mut ArmCore,
    jvm: &mut Jvm,
    entry_class_name: u32,
    _startup_options: u32,
    argument_count: u32,
    arguments: u32,
) -> Result<()> {
    let entry_class_name = String::from_utf8(read_null_terminated_string_bytes(core, entry_class_name)?)
        .map_err(|error| WieError::FatalError(format!("Invalid LGT Java entry class: {error}")))?;
    let mut java_arguments = Vec::with_capacity(argument_count as usize);
    let mut application_class = None;
    for index in 0..argument_count {
        let ptr_argument: u32 = read_generic(core, arguments + index * size_of::<u32>() as u32)?;
        let argument = String::from_utf8(read_null_terminated_string_bytes(core, ptr_argument)?)
            .map_err(|error| WieError::FatalError(format!("Invalid LGT Java startup argument: {error}")))?;
        if index == 0 {
            application_class = Some(argument.clone());
        }
        java_arguments.push(
            JavaLangString::from_rust_string(jvm, &argument)
                .await
                .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?,
        );
    }

    if let Some(application_class) = application_class {
        let loader: Box<dyn ClassInstance> = jvm
            .get_static_field("net/wie/LgtClassLoader", "instance", "Lnet/wie/LgtClassLoader;")
            .await
            .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
        JavaLangClassLoader::load_class(jvm, &loader, &application_class)
            .await
            .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?
            .ok_or_else(|| WieError::FatalError(format!("LGT application class not found: {application_class}")))?;
    }

    let mut argument_array = jvm
        .instantiate_array("Ljava/lang/String;", java_arguments.len())
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    jvm.store_array(&mut argument_array, 0, java_arguments)
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;
    let _: () = jvm
        .invoke_static(&entry_class_name, "main", "([Ljava/lang/String;)V", (argument_array,))
        .await
        .map_err(|JavaError::JavaException(instance)| WieError::JavaException(LgtJvmSupport::class_instance_raw(&*instance)))?;

    Ok(())
}

async fn java_destroy_runtime_context(_core: &mut ArmCore, _: &mut (), runtime_context: u32) -> Result<()> {
    tracing::warn!("stub java_destroy_runtime_context({runtime_context:#x})");

    Ok(())
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc, vec::Vec};
    use core::{
        mem::size_of,
        sync::atomic::{AtomicBool, Ordering},
    };

    use wipi_types::lgt::java::{LgtJavaClass as RawJavaClass, LgtJavaClassLink as RawJavaClassLink};

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{ByteWrite, Result, read_generic, write_generic, write_null_terminated_string_bytes};

    use super::{LgtJvmSupport, java_link_imported_classes};

    #[test]
    fn test_imported_member_link_outputs() -> Result<()> {
        let mut system = System::new(Box::new(TestPlatform::new()), "", "", DefaultTaskRunner);
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let system_clone = system.clone();

        system.spawn(async move || {
            let mut core = ArmCore::new(false, None)?;
            Allocator::init(&mut core)?;
            let mut context = core.save_context();
            let stack = Allocator::alloc(&mut core, 0x100)?;
            context.sp = stack + 0x100;
            core.restore_context(&context);
            let mut jvm = LgtJvmSupport::init(&mut core, &system_clone, None).await?;
            let class_name = "org/kwis/msp/lcdui/Font";
            jvm.resolve_class(class_name).await.unwrap();

            let mut strings = Vec::new();
            for value in [
                class_name,
                "face",
                "I",
                "FACE_SYSTEM",
                "getHeight",
                "()I",
                "getFont",
                "(III)Lorg/kwis/msp/lcdui/Font;",
            ] {
                let address = Allocator::alloc(&mut core, (value.len() + 1) as u32)?;
                write_null_terminated_string_bytes(&mut core, address, value.as_bytes())?;
                strings.push(address);
            }

            let imported_classes = Allocator::alloc(&mut core, (size_of::<u32>() + size_of::<RawJavaClassLink>()) as u32)?;
            write_generic(&mut core, imported_classes, 1u32)?;
            write_generic(
                &mut core,
                imported_classes + size_of::<u32>() as u32,
                RawJavaClassLink {
                    ptr_name: strings[0],
                    instance_field_offset: 0,
                    instance_field_count: 1,
                    static_field_offset: 0,
                    static_field_count: 1,
                    virtual_method_offset: 0,
                    virtual_method_count: 1,
                    interface_method_offset: 0,
                    interface_method_count: 0,
                    non_virtual_method_offset: 0,
                    non_virtual_method_count: 3,
                },
            )?;

            let instance_field_imports = Allocator::alloc(&mut core, 2 * size_of::<u32>() as u32)?;
            write_generic(&mut core, instance_field_imports, strings[1])?;
            write_generic(&mut core, instance_field_imports + size_of::<u32>() as u32, strings[2])?;
            let static_field_imports = Allocator::alloc(&mut core, 2 * size_of::<u32>() as u32)?;
            write_generic(&mut core, static_field_imports, strings[3])?;
            write_generic(&mut core, static_field_imports + size_of::<u32>() as u32, strings[2])?;
            let virtual_method_imports = Allocator::alloc(&mut core, 2 * size_of::<u32>() as u32)?;
            write_generic(&mut core, virtual_method_imports, strings[4])?;
            write_generic(&mut core, virtual_method_imports + size_of::<u32>() as u32, strings[5])?;
            let non_virtual_method_imports = Allocator::alloc(&mut core, 6 * size_of::<u32>() as u32)?;
            core.write_bytes(non_virtual_method_imports, &[0; 6 * size_of::<u32>()])?;
            write_generic(&mut core, non_virtual_method_imports + 4 * size_of::<u32>() as u32, strings[6])?;
            write_generic(&mut core, non_virtual_method_imports + 5 * size_of::<u32>() as u32, strings[7])?;

            let instance_field_word_indices = Allocator::alloc(&mut core, size_of::<u32>() as u32)?;
            let static_field_word_indices = Allocator::alloc(&mut core, size_of::<u32>() as u32)?;
            let virtual_method_indices = Allocator::alloc(&mut core, size_of::<u32>() as u32)?;
            let interface_method_indices = Allocator::alloc(&mut core, size_of::<u32>() as u32)?;
            let non_virtual_method_targets = Allocator::alloc(&mut core, 4 * size_of::<u32>() as u32)?;
            write_generic(&mut core, instance_field_word_indices, 0xa1a1_0000u32)?;
            write_generic(&mut core, static_field_word_indices, 0xb2b2_0000u32)?;
            write_generic(&mut core, virtual_method_indices, 0xc3c3_0000u32)?;
            write_generic(&mut core, interface_method_indices, 0xd4d4_d4d4u32)?;
            for index in 0..4 {
                write_generic(&mut core, non_virtual_method_targets + index * size_of::<u32>() as u32, 0xe5e5_e5e5u32)?;
            }

            java_link_imported_classes(
                &mut core,
                &mut jvm,
                imported_classes,
                instance_field_imports,
                static_field_imports,
                virtual_method_imports,
                0,
                non_virtual_method_imports,
                instance_field_word_indices,
                static_field_word_indices,
                virtual_method_indices,
                interface_method_indices,
                non_virtual_method_targets,
            )
            .await?;

            let instance_word_index = LgtJvmSupport::field_word_index(&jvm, class_name, "face", "I", false)?;
            let static_word_index = LgtJvmSupport::field_word_index(&jvm, class_name, "FACE_SYSTEM", "I", true)?;
            assert_eq!(
                read_generic::<u32, _>(&core, instance_field_word_indices)?,
                0xa1a1_0000 | u32::from(instance_word_index)
            );
            assert_eq!(
                read_generic::<u32, _>(&core, static_field_word_indices)?,
                0xb2b2_0000 | u32::from(static_word_index)
            );

            let virtual_method_index: u16 = read_generic(&core, virtual_method_indices)?;
            assert_eq!(read_generic::<u16, _>(&core, virtual_method_indices + 2)?, 0xc3c3);
            let class = jvm.get_class(class_name).unwrap();
            let ptr_class = LgtJvmSupport::class_definition_raw(&*class.definition);
            let raw_class: RawJavaClass = read_generic(&core, ptr_class)?;
            let virtual_target: u32 = read_generic(&core, raw_class.unk1 + (u32::from(virtual_method_index) + 1) * size_of::<u32>() as u32)?;
            assert_eq!(
                virtual_target,
                LgtJvmSupport::non_virtual_method_target(&jvm, class_name, "getHeight", "()I")?
            );

            let (initialized_class_getter, class_getter) = LgtJvmSupport::class_getter_targets(&jvm, class_name)?;
            assert_eq!(read_generic::<u32, _>(&core, non_virtual_method_targets)?, initialized_class_getter);
            assert_eq!(read_generic::<u32, _>(&core, non_virtual_method_targets + 4)?, class_getter);
            assert_eq!(
                read_generic::<u32, _>(&core, non_virtual_method_targets + 8)?,
                LgtJvmSupport::non_virtual_method_target(&jvm, class_name, "getFont", "(III)Lorg/kwis/msp/lcdui/Font;")?
            );
            assert_eq!(read_generic::<u32, _>(&core, non_virtual_method_targets + 12)?, 0xe5e5_e5e5);
            assert_eq!(read_generic::<u32, _>(&core, interface_method_indices)?, 0xd4d4_d4d4);

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
