use alloc::format;
use core::mem::size_of;

use elf::{ElfBytes, endian::AnyEndian};

use jvm::Jvm;
use wipi_types::lgt::{InitParam1, InitParam2, InitStruct};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, ResultWriter, SvcId};
use wie_util::{Result, WieError, read_generic, write_generic};

use super::{
    SVC_CATEGORY_INIT, SVC_CATEGORY_JAVA_INTERFACE, SVC_CATEGORY_STDLIB, SVC_CATEGORY_WIPIC,
    java::{
        get_java_interface_method,
        interface::{
            handle_java_interface_svc, java_interface_stub, java_interface_unk84, java_load_classes, java_unk0, java_unk5, java_unk9, java_unk11,
            java_unk12,
        },
        native_jvm::{LgtJvmShared, register_app_classes, register_java_trampoline_handler},
    },
    stdlib::register_stdlib_svc_handler,
    svc_ids::InitSvcId,
    wipi_c::register_wipic_svc_handler,
};

fn register_init_svc_handler(core: &mut ArmCore, shared: &LgtJvmShared) -> Result<()> {
    core.register_svc_handler(
        SVC_CATEGORY_INIT,
        handle_init_svc,
        &(SVC_CATEGORY_WIPIC, SVC_CATEGORY_STDLIB, shared.clone()),
    )
}

async fn handle_init_svc(core: &mut ArmCore, (wipic_category, stdlib_category, shared): &mut (u32, u32, LgtJvmShared), id: SvcId) -> Result<()> {
    let (_, lr) = core.read_pc_lr()?;

    // Java-interface handlers that need JVM/System/trampoline access share this.
    let mut java_ctx = shared.clone();

    match InitSvcId::try_from(id)? {
        InitSvcId::GetImportTable => EmulatedFunction::call(&get_import_table, core, &mut ()).await?.write(core, lr),
        InitSvcId::GetImportFunction => get_import_function(core, *wipic_category, *stdlib_category, core.read_param(0)?, core.read_param(1)?)
            .await?
            .write(core, lr),
        InitSvcId::Unk0 => EmulatedFunction::call(&unk0, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaUnk7 => EmulatedFunction::call(&java_unk7, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaUnk1 => EmulatedFunction::call(&java_unk1, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaUnk2 => EmulatedFunction::call(&java_unk2, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaUnk3 => EmulatedFunction::call(&java_unk3, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaInterfaceUnk0 => EmulatedFunction::call(&java_unk0, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaInterfaceUnk12 => EmulatedFunction::call(&java_unk12, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaInterfaceUnk5 => EmulatedFunction::call(&java_unk5, core, &mut java_ctx).await?.write(core, lr),
        InitSvcId::JavaLoadClasses => EmulatedFunction::call(&java_load_classes, core, &mut java_ctx).await?.write(core, lr),
        InitSvcId::JavaUnk9 => EmulatedFunction::call(&java_unk9, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaUnk11 => EmulatedFunction::call(&java_unk11, core, &mut java_ctx).await?.write(core, lr),
        InitSvcId::JavaInterfaceUnk84 => EmulatedFunction::call(&java_interface_unk84, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaInterfaceStub => EmulatedFunction::call(&java_interface_stub, core, &mut ()).await?.write(core, lr),
        InitSvcId::JavaNewObject => java_ctx.alloc_native_object(core)?.write(core, lr),
    }
}

pub async fn load_native(core: &mut ArmCore, system: &mut System, jvm: &Jvm, data: &[u8]) -> Result<()> {
    let LoadedExecutable { entrypoint, data_range } = load_executable(core, data)?;
    register_wipic_svc_handler(core, system, jvm)?;

    // Shared LGT native-JVM runtime (instance registry + native->platform trampolines).
    let shared = LgtJvmShared::new(jvm.clone(), system.clone());
    register_stdlib_svc_handler(core, &shared)?;
    register_java_trampoline_handler(core, &shared)?;
    register_init_svc_handler(core, &shared)?;
    core.register_svc_handler(SVC_CATEGORY_JAVA_INTERFACE, handle_java_interface_svc, &shared)?;

    let ptr_init_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    let ptr_init_param_2 = Allocator::alloc(core, size_of::<InitParam2>() as u32)?;

    let init_param_1 = InitParam1 {
        unk1: [0; 512],
        unk2: [0; 20],
        ptr_init_struct: 0,
    };

    write_generic(core, ptr_init_param_1, init_param_1)?;

    let init_param_2 = InitParam2 {
        fn_get_import_table: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetImportTable)?,
        fn_get_import_function: core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetImportFunction)?,
        fn_unk3: 0,
        fn_unk4: 0,
    };

    write_generic(core, ptr_init_param_2, init_param_2)?;

    tracing::debug!("ptr_init_param_1: {ptr_init_param_1:#x}");
    tracing::debug!("ptr_init_param_2: {ptr_init_param_2:#x}");

    tracing::debug!("Calling entrypoint {entrypoint:#x}");
    let _: () = core.run_function(entrypoint + 1, &[ptr_init_param_1, ptr_init_param_2, 0]).await?;

    let init_param_1: InitParam1 = read_generic(core, ptr_init_param_1)?;

    tracing::debug!("InitStruct: {:#x?}", init_param_1.ptr_init_struct);
    let init_struct: InitStruct = read_generic(core, init_param_1.ptr_init_struct)?;

    // Register the app's native classes with the JVM before the initializer runs
    // (the initializer drives the Java-interface boot, incl. Main.main -> new Game).
    // No-op for WIPI-C clet apps, whose `.data` holds no class descriptors.
    if let Some((data_start, data_end)) = data_range {
        let registered = register_app_classes(jvm, core, &shared, data_start, data_end).await?;
        if !registered.is_empty() {
            tracing::info!("LGT native JVM: registered {} app classes: {registered:?}", registered.len());
        }
    }

    tracing::debug!("Calling initializer at {:#x}", init_struct.fn_init);
    let _: () = core.run_function(init_struct.fn_init, &[]).await?;

    Ok(())
}

async fn get_import_table(_core: &mut ArmCore, _: &mut (), import_table: u32) -> Result<u32> {
    tracing::debug!("get_import_table({import_table:#x})");

    Ok(import_table)
}

async fn get_import_function(core: &mut ArmCore, wipic_category: u32, stdlib_category: u32, import_table: u32, function_index: u32) -> Result<u32> {
    tracing::debug!("get_import_function({import_table:#x}, {function_index})");

    if import_table == 0x1fb {
        return core.make_svc_stub(wipic_category, function_index);
    } else if import_table == 0x64 {
        return get_java_interface_method(core, function_index);
    } else if import_table == 1 {
        return core.make_svc_stub(stdlib_category, function_index);
    }

    Ok(match (import_table, function_index) {
        (0x1f8, 0x16) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::Unk0)?,
        (0x1f8, 0x17) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk7)?,
        (0x1fc, 0x03) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk1)?,
        (0x1ff, 0x03) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk2)?,
        (0x201, 0x03) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::JavaUnk3)?,
        _ => {
            return Err(WieError::FatalError(format!(
                "Unknown import function: {import_table:#x}, {function_index:#x}"
            )));
        }
    })
}

/// The loaded executable's entrypoint plus the `.data` segment range
/// (`start..end`), used to locate the app's native class descriptors.
struct LoadedExecutable {
    entrypoint: u32,
    data_range: Option<(u32, u32)>,
}

fn load_executable(core: &mut ArmCore, data: &[u8]) -> Result<LoadedExecutable> {
    let elf = ElfBytes::<AnyEndian>::minimal_parse(data).map_err(|x| WieError::FatalError(format!("Failed to parse ELF binary.mod: {x}")))?;

    if elf.ehdr.e_machine != elf::abi::EM_ARM {
        return Err(WieError::FatalError(format!("Invalid ELF machine type: {}", elf.ehdr.e_machine)));
    }
    if elf.ehdr.e_type != elf::abi::ET_EXEC {
        return Err(WieError::FatalError(format!("Invalid ELF file type: {}", elf.ehdr.e_type)));
    }
    if elf.ehdr.class != elf::file::Class::ELF32 {
        return Err(WieError::FatalError(format!("Invalid ELF class: {:?}", elf.ehdr.class)));
    }

    let (shdrs_opt, strtab_opt) = elf
        .section_headers_with_strtab()
        .map_err(|x| WieError::FatalError(format!("Failed to read ELF section headers: {x}")))?;
    let shdrs = shdrs_opt.ok_or_else(|| WieError::FatalError("ELF is missing section headers".into()))?;
    let strtab = strtab_opt.ok_or_else(|| WieError::FatalError("ELF is missing section name string table".into()))?;

    let mut data_range = None;

    for shdr in shdrs {
        let section_name = strtab
            .get(shdr.sh_name as usize)
            .map_err(|x| WieError::FatalError(format!("Invalid ELF section name index {}: {x}", shdr.sh_name)))?;

        if shdr.sh_addr != 0 {
            tracing::debug!("Section {section_name} at {:x}", shdr.sh_addr);

            if section_name == ".data" {
                data_range = Some((shdr.sh_addr as u32, (shdr.sh_addr + shdr.sh_size) as u32));
            }

            let data = elf
                .section_data(&shdr)
                .map_err(|x| WieError::FatalError(format!("Failed to read ELF section {section_name}: {x}")))?
                .0;

            core.load(data, shdr.sh_addr as u32, shdr.sh_size as usize)?;
        }
    }

    tracing::debug!("Entrypoint: {:#x}", elf.ehdr.e_entry);

    Ok(LoadedExecutable {
        entrypoint: elf.ehdr.e_entry as u32,
        data_range,
    })
}

async fn unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("clet_unk0({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(())
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

async fn java_unk7(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<u32> {
    tracing::warn!("java_unk7({a0:#x}, {a1:#x}, {a2:#x})");

    // get jar path?

    Ok(0 as _)
}
