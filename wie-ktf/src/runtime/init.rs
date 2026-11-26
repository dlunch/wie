use alloc::{format, string::String};
use core::mem::{offset_of, size_of};
use jvm::Jvm;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_jvm_support::JvmSupport;
use wie_util::{Result, WieError, read_generic, read_null_terminated_string_bytes, write_generic};

use wipi_types::ktf::{ExeInterface, ExeInterfaceFunctions, InitParam0, InitParam3, InitParam4, WipiExe};

use crate::{
    adf::parse_bss_size,
    emulator::IMAGE_BASE,
    runtime::{
        SVC_CATEGORY_INIT,
        java::{
            interface::{get_wipi_jb_interface, java_array_new, java_class_load, java_is_instance, java_new, java_throw, java_throw_instance},
            jvm_support::{KtfJvmSupport, KtfJvmSupportContext, SUPPORT_CONTEXT_BASE},
        },
        svc_ids::InitSvcId,
        wipi_c::{
            interface::{get_user_memory_interface, get_wipic_knl_interface},
            register_wipic_svc_handler,
        },
        with_arm_context,
    },
};

pub fn register_init_svc_handler(core: &mut ArmCore, jvm: &Jvm) -> Result<()> {
    // Apps look this table up repeatedly, so it is built once.
    let user_memory = get_user_memory_interface(core)?;
    core.register_svc_handler(SVC_CATEGORY_INIT, handle_init_svc, &(jvm.clone(), user_memory))
}

async fn handle_init_svc(core: &mut ArmCore, (jvm, user_memory): &mut (Jvm, u32), id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    match InitSvcId::try_from(id)? {
        InitSvcId::GetInterface => get_interface(core, *user_memory)?.write(core, lr),
        InitSvcId::JavaThrow => EmulatedFunction::call(&java_throw, core, jvm).await?.write(core, lr),
        InitSvcId::JavaThrowInstance => EmulatedFunction::call(&java_throw_instance, core, jvm).await?.write(core, lr),
        InitSvcId::JavaIsInstance => EmulatedFunction::call(&java_is_instance, core, jvm).await?.write(core, lr),
        InitSvcId::JavaNew => EmulatedFunction::call(&java_new, core, jvm).await?.write(core, lr),
        InitSvcId::JavaArrayNew => EmulatedFunction::call(&java_array_new, core, jvm).await?.write(core, lr),
        InitSvcId::JavaClassLoad => EmulatedFunction::call(&java_class_load, core, jvm).await?.write(core, lr),
        InitSvcId::Alloc => EmulatedFunction::call(&alloc, core, &mut ()).await?.write(core, lr),
        InitSvcId::VisitGcRoot => Err(WieError::FatalError("Root callback outside garbage collection".into())),
    }
}

pub async fn load_native(
    core: &mut ArmCore,
    system: &mut System,
    jvm: &Jvm,
    filename: &str,
    data: &[u8],
    ptr_class_base: u32,
    ptr_current_jvm_thread_context: u32,
) -> Result<u32> {
    let bss_size = parse_bss_size(filename)?;

    core.load(data, IMAGE_BASE, data.len() + bss_size as usize)?;
    core.prepare_execution().await?;

    register_wipic_svc_handler(core, system, jvm)?;
    register_init_svc_handler(core, jvm)?;

    tracing::debug!("Loaded at {IMAGE_BASE:#x}, size {:#x}, bss {bss_size:#x}", data.len());

    let ptr_wipi_exe = core
        .run_function(IMAGE_BASE + 1, &[bss_size])
        .await
        .map_err(|error| with_arm_context(core, error))?;
    tracing::debug!("Got wipi_exe {ptr_wipi_exe:#x}");

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { safepoint_requested: 0 })?;

    let mut primitive_array_classes = [0; 8];
    for (ptr_class, name) in primitive_array_classes.iter_mut().zip(["[Z", "[C", "[F", "[D", "[B", "[S", "[I", "[J"]) {
        let class = match jvm.resolve_class(name).await {
            Ok(x) => x,
            Err(x) => return Err(JvmSupport::to_wie_err(jvm, x).await),
        };
        *ptr_class = KtfJvmSupport::class_definition_raw(&*class.definition)?;
    }
    let [boolean, char, float, double, byte, short, int, long] = primitive_array_classes;

    let param_3 = InitParam3 {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        boolean,
        char,
        float,
        double,
        byte,
        short,
        int,
        long,
    };

    let ptr_param_3 = Allocator::alloc(core, size_of::<InitParam3>() as u32)?;
    write_generic(core, ptr_param_3, param_3)?;

    let param_4 = InitParam4 {
        fn_get_interface: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetInterface)?,
        fn_java_throw: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaThrow)?,
        fn_java_throw_instance: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaThrowInstance)?,
        unk2: 0,
        fn_java_is_instance: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaIsInstance)?,
        fn_java_new: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaNew)?,
        fn_java_array_new: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaArrayNew)?,
        fn_visit_gc_root: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::VisitGcRoot)?,
        fn_java_class_load: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaClassLoad)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::Alloc)?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let wipi_exe: WipiExe = read_generic(core, ptr_wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    tracing::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_current_jvm_thread_context, ptr_class_base, ptr_param_3, ptr_param_4],
        )
        .await
        .map_err(|error| with_arm_context(core, error))?;

    if result != 0 {
        return Err(WieError::FatalError(format!("Init failed with code {result:#x}")));
    }

    // call init
    let result = core
        .run_function::<u32>(wipi_exe.fn_init, &[])
        .await
        .map_err(|error| with_arm_context(core, error))?;
    if result != 0 {
        return Err(WieError::FatalError(format!("wipi init failed with code {result:#x}")));
    }

    // The enclosing JVM frame roots initialization objects until the native tables are ready.
    write_generic(
        core,
        SUPPORT_CONTEXT_BASE + offset_of!(KtfJvmSupportContext, ptr_visit_gc_root) as u32,
        wipi_exe.fn_visit_gc_roots,
    )?;

    Ok(exe_interface.ptr_functions)
}

fn get_interface(core: &mut ArmCore, user_memory: u32) -> Result<u32> {
    let name = read_null_terminated_string_bytes(core, core.read_param(0)?)?;
    let name = String::from_utf8_lossy(&name);
    tracing::debug!(
        "get_interface({name:?}, {:#x}, {:#x}, {:#x}, {:#x})",
        core.read_param(1)?,
        core.read_param(2)?,
        core.read_param(3)?,
        core.read_param(4)?
    );

    match name.as_ref() {
        "MXUserMemInterf" => Ok(user_memory),
        "WIPIC_knlInterface" => get_wipic_knl_interface(core),
        "WIPI_JBInterface" => get_wipi_jb_interface(core),
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
