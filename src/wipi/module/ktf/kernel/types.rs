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
