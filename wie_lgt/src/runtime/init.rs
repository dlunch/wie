use core::mem::size_of;

use bytemuck::{Pod, Zeroable};
use elf::{endian::AnyEndian, ElfBytes};

use wie_backend::System;
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{read_generic, write_generic, Result};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitStruct {
    unk1: u32,
    unk2: u32,
    ptr_str_init: u32, // pointer to string "init"
    unk3: u32,
    unk4: u32,
    unk5: u32,
    fn_unk1: u32,
    fn_unk2: u32,
    fn_unk3: u32,
    fn_null1: u32,
    fn_null2: u32,
    fn_unk4: u32,
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

pub async fn load_native(core: &mut ArmCore, _system: &mut System, data: &[u8]) -> Result<()> {
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
        fn_get_import_function: core.register_function(get_import_function, &())?,
        fn_unk3: 0,
        fn_unk4: 0,
    };

    write_generic(core, ptr_init_param_2, init_param_2)?;

    tracing::debug!("ptr_init_param_1: {:#x}", ptr_init_param_1);
    tracing::debug!("ptr_init_param_2: {:#x}", ptr_init_param_2);
    let entrypoint = load_executable(core, data)?;

    tracing::debug!("Calling entrypoint {:#x}", entrypoint);
    let _: () = core.run_function(entrypoint + 1, &[ptr_init_param_1, ptr_init_param_2, 0]).await?;

    let init_param_1: InitParam1 = read_generic(core, ptr_init_param_1)?;

    tracing::debug!("InitStruct: {:#x?}", init_param_1.ptr_init_struct);
    let init_struct: InitStruct = read_generic(core, init_param_1.ptr_init_struct)?;

    tracing::debug!("Calling initializer at {:#x}", init_struct.fn_unk1);
    let _: () = core.run_function(init_struct.fn_unk1, &[]).await?;

    Ok(())
}

async fn get_import_table(_core: &mut ArmCore, _: &mut (), import_table: u32) -> Result<u32> {
    tracing::warn!("stub get_import_table({:#x})", import_table);

    Ok(import_table)
}

async fn get_import_function(_core: &mut ArmCore, _: &mut (), import_table: u32, function_index: u32) -> Result<u32> {
    tracing::warn!("stub get_import_function({:#x}, {})", import_table, function_index);

    Ok(1)
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
