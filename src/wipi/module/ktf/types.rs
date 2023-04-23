#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitParam4 {
    pub get_system_function_fn: u32,
    pub get_java_function_fn: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WipiExe {
    pub exe_interface_ptr: u32,
    name_ptr: u32,
    unk1: u32,
    unk2: u32,
    unk1_fn: u32,
    unk2_fn: u32,
    unk3: u32,
    unk4: u32,
    unk3_fn: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExeInterface {
    pub functions_ptr: u32,
    name_ptr: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    pub init_fn: u32,
    get_default_dll_fn: u32,
    unk1_fn: u32,
    unk2_fn: u32,
    unk3_fn: u32,
}
