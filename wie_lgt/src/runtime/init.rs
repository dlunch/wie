use alloc::format;
use core::mem::size_of;

use elf::{ElfBytes, endian::AnyEndian};

use jvm::Jvm;
use wipi_types::lgt::{InitParam1, InitParam2, InitStruct};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore, EmulatedFunction, JumpTo, ResultWriter, SvcId};
use wie_util::{Result, WieError, read_generic, write_generic, write_null_terminated_string_bytes};

use super::{
    SVC_CATEGORY_INIT, SVC_CATEGORY_JAVA_SYSTEM, SVC_CATEGORY_STDLIB, SVC_CATEGORY_WIPIC,
    java::{JavaExceptionState, get_java_interface_method, register_java_system_svc_handler},
    stdlib::register_stdlib_svc_handler,
    svc_ids::{InitSvcId, JavaSystemSvcId},
    wipi_c::register_wipic_svc_handler,
};

fn register_init_svc_handler(core: &mut ArmCore, ptr_jar_path: u32) -> Result<()> {
    core.register_svc_handler(SVC_CATEGORY_INIT, handle_init_svc, &ptr_jar_path)
}

async fn handle_init_svc(core: &mut ArmCore, ptr_jar_path: &mut u32, id: SvcId) -> Result<JumpTo> {
    let (_, lr) = core.read_pc_lr()?;
    match InitSvcId::try_from(id)? {
        InitSvcId::GetImportTable => EmulatedFunction::call(&get_import_table, core, &mut ()).await?.write(core, lr)?,
        InitSvcId::GetImportFunction => get_import_function(core, core.read_param(0)?, core.read_param(1)?)
            .await?
            .write(core, lr)?,
        InitSvcId::Unk0 => EmulatedFunction::call(&unk0, core, &mut ()).await?.write(core, lr)?,
        InitSvcId::GetApplicationJarPath => EmulatedFunction::call(&get_application_jar_path, core, ptr_jar_path)
            .await?
            .write(core, lr)?,
    }

    Ok(JumpTo(lr))
}

pub async fn load_native(
    core: &mut ArmCore,
    system: &mut System,
    jvm: &Jvm,
    exception_state: JavaExceptionState,
    jar_path: &str,
    data: &[u8],
) -> Result<()> {
    let entrypoint = load_executable(core, data)?;
    register_wipic_svc_handler(core, system, jvm)?;
    register_stdlib_svc_handler(core, system)?;
    let ptr_jar_path_value = Allocator::alloc(core, (jar_path.len() + 1) as u32)?;
    write_null_terminated_string_bytes(core, ptr_jar_path_value, jar_path.as_bytes())?;
    let ptr_jar_path = Allocator::alloc(core, size_of::<u32>() as u32)?;
    write_generic(core, ptr_jar_path, ptr_jar_path_value)?;
    register_init_svc_handler(core, ptr_jar_path)?;
    register_java_system_svc_handler(core, jvm, exception_state, ptr_jar_path)?;

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

    tracing::debug!("Calling initializer at {:#x}", init_struct.fn_init);
    let _: () = core.run_function(init_struct.fn_init, &[]).await?;

    Ok(())
}

async fn get_import_table(_core: &mut ArmCore, _: &mut (), import_table: u32) -> Result<u32> {
    tracing::debug!("get_import_table({import_table:#x})");

    Ok(import_table)
}

async fn get_import_function(core: &mut ArmCore, import_table: u32, function_index: u32) -> Result<u32> {
    tracing::debug!("get_import_function({import_table:#x}, {function_index})");

    if import_table == 0x1fb {
        return core.make_svc_stub(SVC_CATEGORY_WIPIC, function_index);
    } else if import_table == 0x64 {
        return get_java_interface_method(core, function_index);
    } else if import_table == 1 && function_index == 0x32 {
        return core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::PendingException);
    } else if import_table == 1 {
        return core.make_svc_stub(SVC_CATEGORY_STDLIB, function_index);
    }

    Ok(match (import_table, function_index) {
        (0x1f8, 0x16) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::Unk0)?,
        (0x1f8, 0x17) => core.make_svc_stub(SVC_CATEGORY_INIT, InitSvcId::GetApplicationJarPath)?,
        (0x1fc, 0x03) => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Unk1)?,
        (0x1ff, 0x03) => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Unk2)?,
        (0x201, 0x03) => core.make_svc_stub(SVC_CATEGORY_JAVA_SYSTEM, JavaSystemSvcId::Unk3)?,
        _ => {
            return Err(WieError::FatalError(format!(
                "Unknown import function: {import_table:#x}, {function_index:#x}"
            )));
        }
    })
}

fn load_executable(core: &mut ArmCore, data: &[u8]) -> Result<u32> {
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

    for shdr in shdrs {
        let section_name = strtab
            .get(shdr.sh_name as usize)
            .map_err(|x| WieError::FatalError(format!("Invalid ELF section name index {}: {x}", shdr.sh_name)))?;

        if shdr.sh_addr != 0 {
            tracing::debug!("Section {section_name} at {:x}", shdr.sh_addr);

            let data = elf
                .section_data(&shdr)
                .map_err(|x| WieError::FatalError(format!("Failed to read ELF section {section_name}: {x}")))?
                .0;

            core.load(data, shdr.sh_addr as u32, shdr.sh_size as usize)?;
        }
    }

    tracing::debug!("Entrypoint: {:#x}", elf.ehdr.e_entry);

    Ok(elf.ehdr.e_entry as u32)
}

async fn unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("clet_unk0({a0:#x}, {a1:#x}, {a2:#x}, {a3:#x})");

    Ok(())
}

async fn get_application_jar_path(core: &mut ArmCore, ptr_jar_path: &mut u32, _a0: u32, _capacity: u32, path_output: u32) -> Result<u32> {
    let jar_path: u32 = read_generic(core, *ptr_jar_path)?;
    write_generic(core, path_output, jar_path)?;
    Ok(0)
}

#[cfg(test)]
mod tests {
    use alloc::{boxed::Box, sync::Arc};
    use core::{
        mem::offset_of,
        sync::atomic::{AtomicBool, Ordering},
    };

    use jvm::runtime::JavaLangString;
    use wipi_types::lgt::java::LgtJavaClassInstance as RawJavaClassInstance;

    use test_utils::TestPlatform;
    use wie_backend::{DefaultTaskRunner, System};
    use wie_core_arm::{Allocator, ArmCore};
    use wie_util::{Result, read_generic, write_generic};

    use super::{get_java_interface_method, register_init_svc_handler};
    use crate::runtime::{LgtJvmSupport, java::register_java_system_svc_handler};

    #[test]
    fn compiler_array_helpers_build_guest_arrays() -> Result<()> {
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

            let (jvm, exception_state) = LgtJvmSupport::init(&mut core, &system_clone, None).await?;
            register_init_svc_handler(&mut core, 0)?;
            register_java_system_svc_handler(&mut core, &jvm, exception_state, 0)?;

            let array = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            let value = JavaLangString::from_rust_string(&jvm, "stored").await.unwrap();
            let ptr_array = array.identity() as u32;
            let ptr_value = value.identity() as u32;
            let target = get_java_interface_method(&mut core, 0xfa)?;
            let _: () = core.run_function(target, &[ptr_array, 0, ptr_value]).await?;

            let ptr_fields: u32 = read_generic(&core, ptr_array + offset_of!(RawJavaClassInstance, ptr_fields) as u32)?;
            assert_eq!(read_generic::<u32, _>(&core, ptr_fields + 4)?, ptr_value);

            let ptr_dimensions = Allocator::alloc(&mut core, 8)?;
            write_generic(&mut core, ptr_dimensions, 2u32)?;
            write_generic(&mut core, ptr_dimensions + 4, 3u32)?;
            let array_class = jvm.resolve_class("[[I").await.unwrap().java_class();
            let target = get_java_interface_method(&mut core, 0x11)?;
            let ptr_multi_array: u32 = core.run_function(target, &[array_class.identity() as u32, ptr_dimensions, 2]).await?;

            let ptr_multi_fields: u32 = read_generic(&core, ptr_multi_array + offset_of!(RawJavaClassInstance, ptr_fields) as u32)?;
            assert_eq!(read_generic::<u32, _>(&core, ptr_multi_fields)?, 2);
            for index in 0..2 {
                let ptr_child: u32 = read_generic(&core, ptr_multi_fields + (index + 1) * 4)?;
                let ptr_child_fields: u32 = read_generic(&core, ptr_child + offset_of!(RawJavaClassInstance, ptr_fields) as u32)?;
                assert_eq!(read_generic::<u32, _>(&core, ptr_child_fields)?, 3);
            }

            done_clone.store(true, Ordering::Relaxed);
            Ok(())
        });

        while !done.load(Ordering::Relaxed) {
            system.tick()?;
        }

        Ok(())
    }
}
