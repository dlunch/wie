use alloc::{format, string::String, vec};
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};
use elf::{endian::AnyEndian, ElfBytes};

use jvm::Jvm;

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, write_generic, ByteRead, ByteWrite, Result, WieError};

use super::wipi_c::get_wipi_c_method;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitStruct {
    unk1: u32,
    fn_init: u32,
    ptr_str_init: u32, // pointer to string "init"
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam1 {
    unk1: [u8; 512],
    unk2: [u8; 20],
    ptr_init_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam2 {
    fn_get_import_table: u32,
    fn_get_import_function: u32,
    fn_unk3: u32,
    fn_unk4: u32,
}

pub async fn load_native(core: &mut ArmCore, system: &mut System, jvm: &Jvm, data: &[u8]) -> Result<()> {
    let entrypoint = load_executable(core, data)?;

    let ptr_init_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    let ptr_init_param_2 = Allocator::alloc(core, size_of::<InitParam2>() as u32)?;

    let init_param_1 = InitParam1 {
        unk1: [0; 512],
        unk2: [0; 20],
        ptr_init_struct: 0,
    };

    write_generic(core, ptr_init_param_1, init_param_1)?;

    let init_param_2 = InitParam2 {
        fn_get_import_table: core.register_function(get_import_table, &())?,
        fn_get_import_function: core.register_function(get_import_function, &(system.clone(), jvm.clone()))?,
        fn_unk3: 0,
        fn_unk4: 0,
    };

    write_generic(core, ptr_init_param_2, init_param_2)?;

    tracing::debug!("ptr_init_param_1: {:#x}", ptr_init_param_1);
    tracing::debug!("ptr_init_param_2: {:#x}", ptr_init_param_2);

    tracing::debug!("Calling entrypoint {:#x}", entrypoint);
    let _: () = core.run_function(entrypoint + 1, &[ptr_init_param_1, ptr_init_param_2, 0]).await?;

    let init_param_1: InitParam1 = read_generic(core, ptr_init_param_1)?;

    tracing::debug!("InitStruct: {:#x?}", init_param_1.ptr_init_struct);
    let init_struct: InitStruct = read_generic(core, init_param_1.ptr_init_struct)?;

    tracing::debug!("Calling initializer at {:#x}", init_struct.fn_init);
    let _: () = core.run_function(init_struct.fn_init, &[]).await?;

    Ok(())
}

async fn get_import_table(_core: &mut ArmCore, _: &mut (), import_table: u32) -> Result<u32> {
    tracing::debug!("get_import_table({:#x})", import_table);

    Ok(import_table)
}

async fn get_import_function(core: &mut ArmCore, (system, jvm): &mut (System, Jvm), import_table: u32, function_index: u32) -> Result<u32> {
    tracing::debug!("get_import_function({:#x}, {})", import_table, function_index);

    if import_table == 0x1fb {
        return get_wipi_c_method(core, system, jvm, function_index);
    }

    Ok(match (import_table, function_index) {
        (0x01, 0x3f6) => core.register_function(java_unk8, &())?,
        (0x01, 0x3fb) => core.register_function(atoi, &())?,
        (0x01, 0x414) => core.register_function(memcpy, &())?,
        (0x01, 0x418) => core.register_function(java_unk4, &())?,
        (0x01, 0x424) => core.register_function(java_unk10, &())?,
        (0x64, 0x03) => core.register_function(java_unk0, &())?,
        (0x64, 0x06) => core.register_function(java_unk12, &())?,
        (0x64, 0x07) => core.register_function(java_unk5, &())?,
        (0x64, 0x14) => core.register_function(java_unk6, &())?,
        (0x64, 0x82) => core.register_function(java_unk9, &())?,
        (0x64, 0x83) => core.register_function(java_unk11, &())?,
        (0x1f8, 0x16) => core.register_function(unk0, &())?,
        (0x1f8, 0x17) => core.register_function(java_unk7, &())?,
        (0x1fc, 0x03) => core.register_function(java_unk1, &())?,
        (0x1ff, 0x03) => core.register_function(java_unk2, &())?,
        (0x201, 0x03) => core.register_function(java_unk3, &())?,
        _ => {
            return Err(WieError::FatalError(format!(
                "Unknown import function: {:#x}, {:#x}",
                import_table, function_index
            )))
        }
    })
}

fn load_executable(core: &mut ArmCore, data: &[u8]) -> Result<u32> {
    let elf = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();

    assert!(elf.ehdr.e_machine == elf::abi::EM_ARM, "Invalid machine type");
    assert!(elf.ehdr.e_type == elf::abi::ET_EXEC, "Invalid file type");
    assert!(elf.ehdr.class == elf::file::Class::ELF32, "Invalid file type");
    assert!(elf.ehdr.e_phnum == 0, "Invalid file type");

    let (shdrs_opt, strtab_opt) = elf.section_headers_with_strtab().unwrap();
    let (shdrs, strtab) = (shdrs_opt.unwrap(), strtab_opt.unwrap());

    for shdr in shdrs {
        let section_name = strtab.get(shdr.sh_name as usize).unwrap();

        if shdr.sh_addr != 0 {
            tracing::debug!("Section {} at {:x}", section_name, shdr.sh_addr);

            let data = elf.section_data(&shdr).unwrap().0;

            core.load(data, shdr.sh_addr as u32, shdr.sh_size as usize)?;
        }
    }

    tracing::debug!("Entrypoint: {:#x}", elf.ehdr.e_entry);

    Ok(elf.ehdr.e_entry as u32)
}

async fn unk0(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("clet_unk0({:#x}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    Ok(())
}

async fn atoi(_core: &mut ArmCore, _: &mut (), str: String) -> Result<u32> {
    tracing::debug!("atoi({})", str);

    Ok(str.parse().unwrap())
}

async fn memcpy(core: &mut ArmCore, _: &mut (), dst: u32, src: u32, size: u32) -> Result<()> {
    tracing::debug!("memcpy({:#x}, {:#x}, {:#x})", dst, src, size);

    let mut memory = vec![0u8; size as usize];

    core.read_bytes(src, &mut memory)?;
    core.write_bytes(dst, &memory)?;

    Ok(())
}

async fn java_unk0(_core: &mut ArmCore, _: &mut (), a0: String, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk0({}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk1(_core: &mut ArmCore, _: &mut (), a0: String, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk1({}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk2(_core: &mut ArmCore, _: &mut (), a0: String, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk2({}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk3(_core: &mut ArmCore, _: &mut (), a0: String, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk3({}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk4(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk4({:#x}, {:#x}, {:#x})", a0, a1, a2);

    Ok(())
}

async fn java_unk5(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32) -> Result<()> {
    tracing::warn!("java_unk5({:#x}, {:#x})", a0, a1);

    // a0: class list

    Ok(())
}

async fn java_unk6(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<()> {
    tracing::warn!("java_unk6({:#x}, {:#x}, {:#x})", a0, a1, a2);

    // a0: importing classes, a1: fields? a2: static fields? a3: methods?

    Ok(())
}

async fn java_unk7(_core: &mut ArmCore, _: &mut (), a0: u32, a1: u32, a2: u32) -> Result<u32> {
    tracing::warn!("java_unk7({:#x}, {:#x}, {:#x})", a0, a1, a2);

    // get jar path?

    Ok(0 as _)
}

async fn java_unk8(_core: &mut ArmCore, _: &mut (), a0: String) -> Result<()> {
    tracing::warn!("java_unk8({})", a0);

    // error exit?

    Ok(())
}

async fn java_unk9(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("java_unk9({:#x})", a0);

    Ok(())
}

async fn java_unk10(core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("java_unk10({:#x})", a0);

    let _: () = core.run_function(a0, &[]).await?;

    Ok(())
}

async fn java_unk11(_core: &mut ArmCore, _: &mut (), a0: String, a1: u32, a2: u32, a3: u32) -> Result<()> {
    tracing::warn!("java_unk11({}, {:#x}, {:#x}, {:#x})", a0, a1, a2, a3);

    Ok(())
}

async fn java_unk12(_core: &mut ArmCore, _: &mut (), a0: u32) -> Result<()> {
    tracing::warn!("java_unk12({:#x})", a0);

    Ok(())
}
