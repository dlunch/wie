#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitParam4 {
    pub fn_get_system_struct: u32,
    pub fn_get_java_function: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub fn_instantiate_java: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WipiExe {
    pub ptr_exe_interface: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    fn_unk1: u32,
    pub fn_init: u32,
    unk3: u32,
    unk4: u32,
    fn_unk3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExeInterface {
    pub ptr_functions: u32,
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
pub struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    pub fn_init: u32,
    fn_get_default_dll: u32,
    fn_unk1: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct WIPICKnlInterface {
    pub unk: [u32; 33],
    pub fn_get_interfaces: u32,
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
