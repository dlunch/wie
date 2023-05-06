use std::mem::size_of;

use crate::core::arm::{allocator::Allocator, ArmCore};

use super::{
    c::interface::get_wipic_knl_interface,
    java::bridge::KtfJavaBridge,
    java::interface::{get_wipi_jb_interface, java_array_new, java_class_load, java_new, java_throw},
    Context,
};

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam0 {
    ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam0Unk {
    unk: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam4 {
    fn_get_interface: u32,
    fn_java_throw: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    fn_java_new: u32,
    fn_java_array_new: u32,
    unk6: u32,
    fn_java_class_load: u32,
    unk7: u32,
    unk8: u32,
    fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1 {
    ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1Unk {
    ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam1UnkUnk {
    unk: [u32; 8],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam2 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    ptr_vtables: [u32; 1], // dynamic size
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam3 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    // java array allocation pool for primitive type
    boolean: u32,
    char: u32,
    float: u32,
    double: u32,
    byte: u32,
    short: u32,
    int: u32,
    long: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WipiExe {
    ptr_exe_interface: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    fn_unk1: u32,
    fn_init: u32,
    unk3: u32,
    unk4: u32,
    fn_unk3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterface {
    ptr_functions: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    fn_init: u32,
    fn_get_default_dll: u32,
    fn_get_class: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

pub struct ModuleInfo {
    pub fn_init: u32,
    pub fn_get_class: u32,
}

pub fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32) -> anyhow::Result<ModuleInfo> {
    Allocator::init(core)?;

    let wipi_exe = core.run_function(base_address + 1, &[bss_size])?;

    log::info!("Got wipi_exe {:#x}", wipi_exe);

    let mut java_bridge = KtfJavaBridge::new(core, context);
    let ptr_classes = java_bridge.load_all_classes()?;
    let vtables = ptr_classes
        .iter()
        .map(|&x| java_bridge.get_ptr_methods(x))
        .collect::<Result<Vec<_>, _>>()?;

    let ptr_unk_struct = Allocator::alloc(core, size_of::<InitParam0Unk>() as u32)?;
    core.write(ptr_unk_struct, InitParam0Unk { unk: 0 })?;

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    core.write(ptr_param_0, InitParam0 { ptr_unk_struct })?;

    let ptr_unk_struct = Allocator::alloc(core, size_of::<InitParam1UnkUnk>() as u32)?;
    core.write(ptr_unk_struct, InitParam1UnkUnk { unk: [0; 8] })?;

    let ptr_unk_struct = Allocator::alloc(core, size_of::<InitParam1Unk>() as u32)?;
    core.write(ptr_unk_struct, InitParam1Unk { ptr_unk_struct })?;

    let ptr_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    core.write(ptr_param_1, InitParam1 { ptr_unk_struct })?;

    let ptr_param_2 = Allocator::alloc(core, (size_of::<InitParam2>() + (ptr_classes.len() - 1) * 4) as u32)?;
    core.write(
        ptr_param_2,
        InitParam2 {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_vtables: [0; 1],
        },
    )?;
    let mut cursor = 12;
    for vtable in vtables {
        core.write(ptr_param_2 + cursor, vtable)?;

        cursor += 4;
    }

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
    core.write(ptr_param_3, param_3)?;

    let param_4 = InitParam4 {
        fn_get_interface: core.register_function(get_interface, context)?,
        fn_java_throw: core.register_function(java_throw, context)?,
        unk1: 0,
        unk2: 0,
        unk3: 0,
        fn_java_new: core.register_function(java_new, context)?,
        fn_java_array_new: core.register_function(java_array_new, context)?,
        unk6: 0,
        fn_java_class_load: core.register_function(java_class_load, context)?,
        unk7: 0,
        unk8: 0,
        fn_unk3: core.register_function(init_unk3, context)?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    core.write(ptr_param_4, param_4)?;

    let wipi_exe = core.read::<WipiExe>(wipi_exe)?;
    let exe_interface = core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions = core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

    log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core.run_function(
        exe_interface_functions.fn_init,
        &[ptr_param_0, ptr_param_1, ptr_param_2, ptr_param_3, ptr_param_4],
    )?;
    if result != 0 {
        return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
    }

    Ok(ModuleInfo {
        fn_init: wipi_exe.fn_init,
        fn_get_class: exe_interface_functions.fn_get_class,
    })
}

fn get_interface(core: &mut ArmCore, context: &Context, r#struct: String) -> anyhow::Result<u32> {
    log::debug!("get_interface({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, context),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, context),
        _ => {
            log::warn!("Unknown {}", r#struct);
            log::warn!("Register dump\n{}", core.dump_regs()?);

            Ok(0)
        }
    }
}

fn init_unk3(core: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    // calloc??
    log::debug!("init_unk3({}, {})", a0, a1);

    log::debug!("\n{}", core.dump_regs()?);

    Allocator::alloc(core, a0 * a1)
}
