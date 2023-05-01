#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitParam4 {
    pub get_system_struct_fn: u32,
    pub get_java_function_fn: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub instantiate_java_fn: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WipiExe {
    pub exe_interface_ptr: u32,
    name_ptr: u32,
    unk1: u32,
    unk2: u32,
    unk1_fn: u32,
    pub init_fn: u32,
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

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WIPICKnlInterface {
    pub unk: [u32; 33],
    pub get_interfaces_fn: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WIPICInterface {
    pub interface_0: u32,
    pub interface_1: u32,
    pub interface_2: u32,
    pub interface_3: u32,
    pub interface_4: u32,
    pub interface_5: u32,
    pub interface_6: u32,
    pub interface_7: u32,
    pub interface_8: u32,
    pub interface_9: u32,
    pub interface_10: u32,
    pub interface_11: u32,
    pub interface_12: u32,
}
