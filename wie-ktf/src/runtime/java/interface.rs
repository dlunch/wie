use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::mem::{offset_of, size_of};

use jvm::{Jvm, runtime::JavaLangString};
use wipi_types::ktf::java::{JavaClass as RawJavaClass, WIPIJBInterface};

use wie_core_arm::{Allocator, ArmCore, ArmCoreContext, EmulatedFunction, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{ByteRead, Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use crate::runtime::java::jvm_support::{
    JavaClassDefinition, JavaClassInstance, JavaMethod, JavaMethodResult, JavaMethodRunResult, KtfJvmSupport, KtfJvmSupportContext, KtfJvmWord,
    SUPPORT_CONTEXT_BASE,
};
use crate::runtime::{SVC_CATEGORY_JAVA_INTERFACE, svc_ids::JavaSvcId};

const CLASS_INITIALIZED: u16 = 8;

pub fn register_java_interface_svc_handler(core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_JAVA_INTERFACE, handle_java_interface_svc, jvm)
}

async fn handle_java_interface_svc(core: &mut ArmCore, jvm: &mut Jvm, id: SvcId) -> Result<JavaMethodResult> {
    let (_, lr) = core.read_pc_lr()?;

    match JavaSvcId::try_from(id)? {
        JavaSvcId::JavaJump1 => return EmulatedFunction::call(&java_jump_1, core, &mut ()).await,
        JavaSvcId::JavaJump2 => return EmulatedFunction::call(&java_jump_2, core, &mut ()).await,
        JavaSvcId::JavaJump3 => return EmulatedFunction::call(&java_jump_3, core, &mut ()).await,
        JavaSvcId::GetJavaMethod => EmulatedFunction::call(&get_java_method, core, &mut ()).await?.write(core, lr),
        JavaSvcId::GetField => EmulatedFunction::call(&get_field, core, &mut ()).await?.write(core, lr),
        JavaSvcId::Safepoint => EmulatedFunction::call(&safepoint, core, &mut ()).await?.write(core, lr),
        JavaSvcId::StackOverflow => EmulatedFunction::call(&stack_overflow, core, &mut ()).await?.write(core, lr),
        JavaSvcId::MonitorEnter => EmulatedFunction::call(&monitor_enter, core, jvm).await?.write(core, lr),
        JavaSvcId::MonitorExit => EmulatedFunction::call(&monitor_exit, core, jvm).await?.write(core, lr),
        JavaSvcId::RegisterClass => EmulatedFunction::call(&register_class, core, jvm).await?.write(core, lr),
        JavaSvcId::RegisterJavaString => EmulatedFunction::call(&register_java_string, core, jvm).await?.write(core, lr),
        JavaSvcId::CallNative => return EmulatedFunction::call(&call_native, core, &mut ()).await,
    }?;

    Ok(JavaMethodResult::new(vec![], None))
}

pub fn get_wipi_jb_interface(core: &mut ArmCore) -> Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_java_jump_1: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump1)?,
        fn_java_jump_2: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump2)?,
        fn_java_jump_3: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::JavaJump3)?,
        fn_get_java_method: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::GetJavaMethod)?,
        fn_get_field: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::GetField)?,
        fn_safepoint: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::Safepoint)?,
        fn_stack_overflow: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::StackOverflow)?,
        fn_monitor_enter: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::MonitorEnter)?,
        fn_monitor_exit: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::MonitorExit)?,
        fn_register_class: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::RegisterClass)?,
        fn_register_java_string: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::RegisterJavaString)?,
        fn_call_native: core.make_svc_stub(SVC_CATEGORY_JAVA_INTERFACE, JavaSvcId::CallNative)?,
    };

    let address = Allocator::alloc(core, size_of::<WIPIJBInterface>() as u32)?;
    write_generic(core, address, interface)?;

    Ok(address)
}

pub async fn java_class_load(core: &mut ArmCore, jvm: &mut Jvm, ptr_target: u32, ptr_name: u32) -> Result<u32> {
    tracing::trace!("load_java_class({ptr_target:#x}, {ptr_name:#x})");

    let name_bytes = read_null_terminated_string_bytes(core, ptr_name)?;
    let name = String::from_utf8_lossy(&name_bytes).to_string();
    let _frame = jvm.push_native_frame(&[]);
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

    let error_bytes = read_null_terminated_string_bytes(core, ptr_error)?;
    let error = String::from_utf8_lossy(&error_bytes).to_string();

    let support: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let _frame = (support.ptr_visit_gc_root != 0).then(|| jvm.push_native_frame(&[]));
    let exception = match jvm.new_class(&error, "()V", ()).await {
        Ok(x) => x,
        Err(x) => return Err(JvmSupport::to_wie_err(jvm, x).await),
    };

    JavaMethod::handle_exception(core, jvm, exception).await
}

pub async fn java_throw_instance(core: &mut ArmCore, jvm: &mut Jvm, ptr_exception: u32, a1: u32) -> Result<JavaMethodResult> {
    tracing::debug!("java_throw_instance({ptr_exception:#x}, {a1})");

    let exception = Box::new(JavaClassInstance::from_raw(ptr_exception, core));

    JavaMethod::handle_exception(core, jvm, exception).await
}

fn map_exception_unwind(core: &mut ArmCore, caller: &ArmCoreContext, context_base: u32, target: u32, next_pc: u32) -> Result<JavaMethodResult> {
    let caller_sp = caller.sp;
    let handler_sp: u32 = read_generic(core, context_base + 9 * 4)?;
    tracing::trace!(
        "Checking Java exception unwind boundary: caller_sp={caller_sp:#x}, handler_sp={handler_sp:#x}, next_pc={next_pc:#x}, target={target:#x}"
    );

    if caller_sp != handler_sp {
        return Err(WieError::JavaExceptionUnwind {
            context_base,
            target,
            next_pc,
        });
    }

    core.restore_context(caller);
    Ok(JavaMethodResult::new(vec![context_base, target], Some(next_pc)))
}

fn map_jump_result(core: &mut ArmCore, caller: &ArmCoreContext, result: core::result::Result<u32, WieError>) -> Result<JavaMethodResult> {
    match result {
        Ok(result) => Ok(JavaMethodResult::new(vec![result], None)),
        Err(WieError::JavaExceptionUnwind {
            context_base,
            target,
            next_pc,
        }) => map_exception_unwind(core, caller, context_base, target, next_pc),
        Err(err) => Err(err),
    }
}

async fn get_java_method(core: &mut ArmCore, _: &mut (), ptr_class: u32, ptr_fullname: u32) -> Result<u32> {
    let fullname = KtfJvmSupport::read_name(core, ptr_fullname)?;

    tracing::debug!("get_java_method({ptr_class:#x}, {fullname})");

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let method = find_java_method(&class, fullname.name(), fullname.descriptor()).await?;
    if method.is_none() {
        return Err(WieError::FatalError(format!("Method {fullname} not found from {}", class.name()?)));
    }
    let method = method.unwrap();
    tracing::trace!("get_java_method result {:#x}", method.ptr_raw);

    Ok(method.ptr_raw)
}

#[allow(clippy::double_must_use)] // temporary until https://github.com/rust-lang/rust-clippy/issues/17529 fix lands
#[async_recursion::async_recursion]
async fn find_java_method(class: &JavaClassDefinition, name: &str, descriptor: &str) -> Result<Option<JavaMethod>> {
    // A class cannot declare a static and an instance method with the same name and descriptor.
    let method = match class.method(name, descriptor, false)? {
        Some(x) => Some(x),
        None => class.method(name, descriptor, true)?,
    };

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

    let caller = core.save_context();
    let result = core.run_function::<u32>(address, &[arg1, 0, 0]).await;
    map_jump_result(core, &caller, result)
}

async fn register_class(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<()> {
    tracing::trace!("register_class({ptr_class:#x})");

    let class: JavaClassDefinition = KtfJvmSupport::class_from_raw(core, ptr_class);
    let class_name = class.name()?;
    let _frame = jvm.push_native_frame(&[]);
    if !jvm.has_class(&class_name) {
        let ktf_class_loader = jvm
            .get_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;")
            .await
            .unwrap();

        let result = jvm.register_class(Box::new(class), Some(ktf_class_loader)).await;
        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(jvm, x).await);
        }
    }

    // KTF AOT also calls this entry point to initialize classes before static field access.
    let class = jvm.resolve_class(&class_name).await.unwrap();
    jvm.ensure_initialized(&class).await.unwrap();

    // AOT code skips this call once the class is marked initialized.
    let flag_address = ptr_class + offset_of!(RawJavaClass, unk_flag) as u32;
    let flag: u16 = read_generic(core, flag_address)?;
    write_generic(core, flag_address, flag | CLASS_INITIALIZED)?;

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

    let rust_string = String::from_utf16_lossy(&bytes_u16);

    let support: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let _frame = (support.ptr_visit_gc_root != 0).then(|| jvm.push_native_frame(&[]));
    let instance = JavaLangString::from_rust_string(jvm, &rust_string).await.unwrap();

    Ok(KtfJvmSupport::class_instance_raw(&instance) as _)
}

async fn get_field(core: &mut ArmCore, _: &mut (), ptr_class: u32, field_name: u32) -> Result<u32> {
    tracing::debug!("get_field({ptr_class:#x}, {field_name:#x})");

    let field_name = KtfJvmSupport::read_name(core, field_name)?;

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let field = class.field(field_name.name(), field_name.descriptor(), true)?;
    let field = if field.is_none() {
        class.field(field_name.name(), field_name.descriptor(), false)?
    } else {
        field
    };

    if let Some(x) = field {
        Ok(x.ptr_raw)
    } else {
        Err(WieError::FatalError(format!("Field {field_name} not found from {}", class.name()?)))
    }
}

async fn safepoint(_: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub safepoint({a0:#x}, {a1:#x})");

    Ok(0)
}

async fn stack_overflow(_: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<u32> {
    tracing::warn!("stub stack_overflow({a0:#x}, {a1:#x})");

    Ok(0)
}

async fn monitor_enter(core: &mut ArmCore, jvm: &mut Jvm, ptr_raw: u32) -> Result<u32> {
    tracing::debug!("monitor_enter({ptr_raw:#x})");

    let instance = JavaClassInstance::from_raw(ptr_raw, core);
    jvm.monitor_enter(&instance).await.unwrap();

    Ok(0)
}

async fn monitor_exit(core: &mut ArmCore, jvm: &mut Jvm, ptr_raw: u32) -> Result<u32> {
    tracing::debug!("monitor_exit({ptr_raw:#x})");

    let instance = JavaClassInstance::from_raw(ptr_raw, core);
    jvm.monitor_exit(&instance).await.unwrap();

    Ok(0)
}

async fn call_native(core: &mut ArmCore, _: &mut (), address: u32, ptr_data: u32) -> Result<JavaMethodResult> {
    tracing::trace!("call_native({address:#x}, {ptr_data:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    let caller = core.save_context();
    let result = match core.run_function::<JavaMethodRunResult>(address, &[ptr_data]).await {
        Ok(result) => result,
        Err(WieError::JavaExceptionUnwind {
            context_base,
            target,
            next_pc,
        }) => return map_exception_unwind(core, &caller, context_base, target, next_pc),
        Err(err) => return Err(err),
    };

    // The caller reads one or two result words back from the argument area.
    write_generic(core, ptr_data, [result.result, result.result_high])?;

    Ok(JavaMethodResult::new(vec![ptr_data], None))
}

async fn java_jump_2(core: &mut ArmCore, _: &mut (), arg1: u32, arg2: u32, address: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_2({arg1:#x}, {arg2:#x}, {address:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    let caller = core.save_context();
    let result = core.run_function::<u32>(address, &[arg1, arg2, 0]).await;
    map_jump_result(core, &caller, result)
}

async fn java_jump_3(core: &mut ArmCore, _: &mut (), arg1: u32, arg2: u32, arg3: u32, address: u32) -> Result<JavaMethodResult> {
    tracing::trace!("java_jump_3({arg1:#x}, {arg2:#x}, {arg3:#x}, {address:#x})");

    if address == 0 {
        return Err(WieError::FatalError("jump native address is null".to_string()));
    }

    let caller = core.save_context();
    let result = core.run_function::<u32>(address, &[arg1, arg2, arg3]).await;
    map_jump_result(core, &caller, result)
}

pub async fn java_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32) -> Result<u32> {
    tracing::trace!("java_new({ptr_class:#x})");

    let class = KtfJvmSupport::class_from_raw(core, ptr_class);
    let class_name = class.name()?;

    let support: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let _frame = (support.ptr_visit_gc_root != 0).then(|| jvm.push_native_frame(&[]));
    let result = jvm.instantiate_class(&class_name).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    let raw = KtfJvmSupport::class_instance_raw(&result.unwrap());

    Ok(raw)
}

pub async fn java_array_new(core: &mut ArmCore, jvm: &mut Jvm, ptr_array_class: u32, count: u32) -> Result<u32> {
    tracing::trace!("java_array_new({ptr_array_class:#x}, {count:#x})");

    let array_class_name = KtfJvmSupport::class_from_raw(core, ptr_array_class).name()?;
    let element_type_name = &array_class_name[1..];
    let support: KtfJvmSupportContext = read_generic(core, SUPPORT_CONTEXT_BASE)?;
    let _frame = (support.ptr_visit_gc_root != 0).then(|| jvm.push_native_frame(&[]));
    let result = jvm.instantiate_array(element_type_name, count as _).await;
    if let Err(x) = result {
        return Err(JvmSupport::to_wie_err(jvm, x).await);
    }

    let raw = KtfJvmSupport::class_instance_raw(&result.unwrap());

    Ok(raw)
}

pub async fn java_is_instance(core: &mut ArmCore, jvm: &mut Jvm, ptr_class: u32, ptr_instance: u32) -> Result<u32> {
    tracing::debug!("java_is_instance({ptr_class:#x}, {ptr_instance:#x})");

    let instance = JavaClassInstance::from_raw(ptr_instance, core);
    let class = JavaClassDefinition::from_raw(ptr_class, core);
    let result = jvm.is_instance(&instance, &class.name()?);

    Ok(if result { 1 } else { 0 })
}
