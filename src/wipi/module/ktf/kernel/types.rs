#[repr(C)]
#[derive(Clone, Copy)]
pub struct InitParam4 {
    pub fn_get_interface: u32,
    pub fn_unk1: u32,
    pub unk1: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
    pub unk6: u32,
    pub fn_load_java_class: u32,
    pub unk7: u32,
    pub unk8: u32,
    pub fn_unk3: u32,
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
    pub fn_get_class: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JavaClass {
    pub ptr_next: u32,
    pub unk1: u32,
    pub ptr_descriptor: u32,
    pub unk2: u32,
    pub unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JavaClassDescriptor {
    pub ptr_name: u32,
    pub unk1: u32,
    pub parent_class: u32,
    pub ptr_methods: u32,
    pub ptr_interfaces: u32,
    pub ptr_properties: u32,
    pub unk3: u32,
    pub unk4: u32,
    pub unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JavaMethod {
    pub fn_body: u32,
    pub ptr_class: u32,
    pub unk1: u32,
    pub ptr_name: u32,
    pub unk2: u32,
    pub unk3: u32,
    pub unk4: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct JavaClassInstance {
    pub ptr_class: u32,
}
