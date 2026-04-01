use alloc::{collections::BTreeMap, format, string::String, sync::Arc};
use core::mem::size_of;
use jvm::Jvm;

use spin::Mutex;
use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcCategory, SvcHandle};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use wipi_types::ktf::{ExeInterface, ExeInterfaceFunctions, InitParam0, InitParam1, InitParam3, InitParam4, WipiExe};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{
        java::interface::{get_wipi_jb_interface, java_array_new, java_check_type, java_class_load, java_new, java_throw},
        svc_ids::InitSvcId,
        wipi_c::{WIPICSvcFunctions, interface::get_wipic_knl_interface, register_wipic_svc_handler},
    },
};

#[derive(Clone)]
pub(crate) struct KtfInitSvcContext {
    pub(crate) system: System,
    pub(crate) jvm: Jvm,
    pub(crate) init_handle: SvcHandle,
    pub(crate) wipic_handle: SvcHandle,
    pub(crate) wipic_functions: WIPICSvcFunctions,
}

pub(crate) fn register_init_svc_handler(
    core: &mut ArmCore,
    system: &System,
    jvm: &Jvm,
    wipic_handle: SvcHandle,
    wipic_functions: WIPICSvcFunctions,
) -> Result<SvcHandle> {
    let context = KtfInitSvcContext {
        system: system.clone(),
        jvm: jvm.clone(),
        init_handle: SvcHandle::new(SvcCategory::Init),
        wipic_handle,
        wipic_functions,
    };

    core.register_svc_handler(SvcCategory::Init, handle_init_svc, &context)
}

async fn handle_init_svc(core: &mut ArmCore, context: &mut KtfInitSvcContext, id: u32) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    match InitSvcId::try_from(id)? {
        InitSvcId::GetInterface => EmulatedFunction::call(&get_interface, core, context).await?.write(core, lr),
        InitSvcId::JavaThrow => EmulatedFunction::call(&java_throw, core, context).await?.write(core, lr),
        InitSvcId::JavaCheckType => EmulatedFunction::call(&java_check_type, core, context).await?.write(core, lr),
        InitSvcId::JavaNew => EmulatedFunction::call(&java_new, core, context).await?.write(core, lr),
        InitSvcId::JavaArrayNew => EmulatedFunction::call(&java_array_new, core, context).await?.write(core, lr),
        InitSvcId::JavaClassLoad => EmulatedFunction::call(&java_class_load, core, context).await?.write(core, lr),
        InitSvcId::Alloc => EmulatedFunction::call(&alloc, core, context).await?.write(core, lr),
        InitSvcId::JavaJump1 => EmulatedFunction::call(&crate::runtime::java::interface::java_jump_1, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JavaJump2 => EmulatedFunction::call(&crate::runtime::java::interface::java_jump_2, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JavaJump3 => EmulatedFunction::call(&crate::runtime::java::interface::java_jump_3, core, context)
            .await?
            .write(core, lr),
        InitSvcId::GetJavaMethod => EmulatedFunction::call(&crate::runtime::java::interface::get_java_method, core, context)
            .await?
            .write(core, lr),
        InitSvcId::GetField => EmulatedFunction::call(&crate::runtime::java::interface::get_field, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JbUnk4 => EmulatedFunction::call(&crate::runtime::java::interface::jb_unk4, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JbUnk5 => EmulatedFunction::call(&crate::runtime::java::interface::jb_unk5, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JbUnk7 => EmulatedFunction::call(&crate::runtime::java::interface::jb_unk7, core, context)
            .await?
            .write(core, lr),
        InitSvcId::JbUnk8 => EmulatedFunction::call(&crate::runtime::java::interface::jb_unk8, core, context)
            .await?
            .write(core, lr),
        InitSvcId::RegisterClass => EmulatedFunction::call(&crate::runtime::java::interface::register_class, core, context)
            .await?
            .write(core, lr),
        InitSvcId::RegisterJavaString => EmulatedFunction::call(&crate::runtime::java::interface::register_java_string, core, context)
            .await?
            .write(core, lr),
        InitSvcId::CallNative => EmulatedFunction::call(&crate::runtime::java::interface::call_native, core, context)
            .await?
            .write(core, lr),
    }
}
pub async fn load_native(
    core: &mut ArmCore,
    system: &mut System,
    jvm: &Jvm,
    filename: &str,
    data: &[u8],
    ptr_jvm_context: u32,
    ptr_jvm_exception_context: u32,
) -> Result<ExeInterfaceFunctions> {
    let bss_start = filename.find("client.bin").unwrap() + 10;
    let bss_size = filename[bss_start..].parse::<u32>().unwrap();

    core.load(data, IMAGE_BASE, data.len() + bss_size as usize)?;

    let wipic_functions = Arc::new(Mutex::new(BTreeMap::new()));
    let wipic_handle = register_wipic_svc_handler(core, &wipic_functions)?;
    let init_handle = register_init_svc_handler(core, system, jvm, wipic_handle, wipic_functions.clone())?;

    tracing::debug!("Loaded at {IMAGE_BASE:#x}, size {:#x}, bss {bss_size:#x}", data.len());

    let wipi_exe = core.run_function(IMAGE_BASE + 1, &[bss_size]).await?;
    tracing::debug!("Got wipi_exe {wipi_exe:#x}");

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { unk: 0 })?;

    let ptr_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    write_generic(core, ptr_param_1, InitParam1 { ptr_jvm_exception_context })?;

    let param_3 = InitParam3 {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        boolean: b'Z' as u32,
        char: b'C' as u32,
        float: b'F' as u32,
        double: b'D' as u32,
        byte: b'B' as u32,
        short: b'S' as u32,
        int: b'I' as u32,
        long: b'J' as u32,
    };

    let ptr_param_3 = Allocator::alloc(core, size_of::<InitParam3>() as u32)?;
    write_generic(core, ptr_param_3, param_3)?;

    let param_4 = InitParam4 {
        fn_get_interface: core.make_svc_stub(init_handle, InitSvcId::GetInterface as u32)?,
        fn_java_throw: core.make_svc_stub(init_handle, InitSvcId::JavaThrow as u32)?,
        unk1: 0,
        unk2: 0,
        fn_java_check_type: core.make_svc_stub(init_handle, InitSvcId::JavaCheckType as u32)?,
        fn_java_new: core.make_svc_stub(init_handle, InitSvcId::JavaNew as u32)?,
        fn_java_array_new: core.make_svc_stub(init_handle, InitSvcId::JavaArrayNew as u32)?,
        unk6: 0,
        fn_java_class_load: core.make_svc_stub(init_handle, InitSvcId::JavaClassLoad as u32)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.make_svc_stub(init_handle, InitSvcId::Alloc as u32)?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let wipi_exe: WipiExe = read_generic(core, wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    tracing::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_param_1, ptr_jvm_context, ptr_param_3, ptr_param_4],
        )
        .await?;

    if result != 0 {
        return Err(WieError::FatalError(format!("Init failed with code {result:#x}")));
    }

    // call init
    let result = core.run_function::<u32>(wipi_exe.fn_init, &[]).await?;
    if result != 0 {
        return Err(WieError::FatalError(format!("wipi init failed with code {result:#x}")));
    }

    Ok(exe_interface_functions)
}

async fn get_interface(core: &mut ArmCore, context: &mut KtfInitSvcContext, ptr_name: u32) -> Result<u32> {
    tracing::trace!("get_interface({ptr_name:#x})");

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?).unwrap();

    match name.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(
            core,
            &mut context.system,
            &context.jvm,
            context.wipic_handle,
            context.wipic_functions.clone(),
        ),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, &context.jvm, context.init_handle),
        _ => {
            tracing::warn!("Unknown {name}");

            Ok(0)
        }
    }
}

async fn alloc(core: &mut ArmCore, _: &mut KtfInitSvcContext, a0: u32) -> Result<u32> {
    tracing::trace!("alloc({a0})");

    Allocator::alloc(core, a0)
}
