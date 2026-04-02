use alloc::{collections::BTreeMap, format, string::String, sync::Arc};
use core::mem::size_of;
use jvm::Jvm;

use spin::Mutex;
use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcCategory, SvcId};
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use wipi_types::ktf::{ExeInterface, ExeInterfaceFunctions, InitParam0, InitParam1, InitParam3, InitParam4, WipiExe};

use crate::{
    emulator::IMAGE_BASE,
    runtime::{
        java::interface::{
            call_native, get_field, get_java_method, get_wipi_jb_interface, java_array_new, java_check_type, java_class_load, java_jump_1,
            java_jump_2, java_jump_3, java_new, java_throw, jb_unk4, jb_unk5, jb_unk7, jb_unk8, register_class, register_java_string,
        },
        svc_ids::InitSvcId,
        wipi_c::{WIPICSvcFunctions, interface::get_wipic_knl_interface, register_wipic_svc_handler},
    },
};

pub(crate) fn register_init_svc_handler(core: &mut ArmCore, system: &System, jvm: &Jvm, wipic_functions: WIPICSvcFunctions) -> Result<()> {
    core.register_svc_handler(SvcCategory::Init, handle_init_svc, &(system.clone(), jvm.clone(), wipic_functions))
}

async fn handle_init_svc(core: &mut ArmCore, (system, jvm, wipic_functions): &mut (System, Jvm, WIPICSvcFunctions), id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    match InitSvcId::try_from(id)? {
        InitSvcId::GetInterface => get_interface(core, system, jvm, wipic_functions.clone(), core.read_param(0)?)
            .await?
            .write(core, lr),
        InitSvcId::JavaThrow => EmulatedFunction::call(&java_throw, core, jvm).await?.write(core, lr),
        InitSvcId::JavaCheckType => EmulatedFunction::call(&java_check_type, core, jvm).await?.write(core, lr),
        InitSvcId::JavaNew => EmulatedFunction::call(&java_new, core, jvm).await?.write(core, lr),
        InitSvcId::JavaArrayNew => EmulatedFunction::call(&java_array_new, core, jvm).await?.write(core, lr),
        InitSvcId::JavaClassLoad => EmulatedFunction::call(&java_class_load, core, jvm).await?.write(core, lr),
        InitSvcId::Alloc => EmulatedFunction::call(&alloc, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaJump1 => EmulatedFunction::call(&java_jump_1, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaJump2 => EmulatedFunction::call(&java_jump_2, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaJump3 => EmulatedFunction::call(&java_jump_3, core, &mut ()).await?.write(core, lr),
        InitSvcId::GetJavaMethod => EmulatedFunction::call(&get_java_method, core, &mut ()).await?.write(core, lr),
        InitSvcId::GetField => EmulatedFunction::call(&get_field, core, &mut ()).await?.write(core, lr),
        InitSvcId::JbUnk4 => EmulatedFunction::call(&jb_unk4, core, &mut ()).await?.write(core, lr),
        InitSvcId::JbUnk5 => EmulatedFunction::call(&jb_unk5, core, &mut ()).await?.write(core, lr),
        InitSvcId::JbUnk7 => EmulatedFunction::call(&jb_unk7, core, &mut ()).await?.write(core, lr),
        InitSvcId::JbUnk8 => EmulatedFunction::call(&jb_unk8, core, &mut ()).await?.write(core, lr),
        InitSvcId::RegisterClass => EmulatedFunction::call(&register_class, core, jvm).await?.write(core, lr),
        InitSvcId::RegisterJavaString => EmulatedFunction::call(&register_java_string, core, jvm).await?.write(core, lr),
        InitSvcId::CallNative => EmulatedFunction::call(&call_native, core, &mut ()).await?.write(core, lr),
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
    register_wipic_svc_handler(core, &wipic_functions)?;
    register_init_svc_handler(core, system, jvm, wipic_functions.clone())?;

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
        fn_get_interface: core.make_svc_stub(SvcCategory::Init, InitSvcId::GetInterface as u32)?,
        fn_java_throw: core.make_svc_stub(SvcCategory::Init, InitSvcId::JavaThrow as u32)?,
        unk1: 0,
        unk2: 0,
        fn_java_check_type: core.make_svc_stub(SvcCategory::Init, InitSvcId::JavaCheckType as u32)?,
        fn_java_new: core.make_svc_stub(SvcCategory::Init, InitSvcId::JavaNew as u32)?,
        fn_java_array_new: core.make_svc_stub(SvcCategory::Init, InitSvcId::JavaArrayNew as u32)?,
        unk6: 0,
        fn_java_class_load: core.make_svc_stub(SvcCategory::Init, InitSvcId::JavaClassLoad as u32)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.make_svc_stub(SvcCategory::Init, InitSvcId::Alloc as u32)?,
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

async fn get_interface(core: &mut ArmCore, system: &mut System, jvm: &Jvm, wipic_functions: WIPICSvcFunctions, ptr_name: u32) -> Result<u32> {
    tracing::trace!("get_interface({ptr_name:#x})");

    let name = String::from_utf8(read_null_terminated_string_bytes(core, ptr_name)?).unwrap();

    match name.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, system, jvm, wipic_functions),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, jvm),
        _ => {
            tracing::warn!("Unknown {name}");

            Ok(0)
        }
    }
}

async fn alloc(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<u32> {
    tracing::trace!("alloc({a0})");

    Allocator::alloc(core, a0)
}
