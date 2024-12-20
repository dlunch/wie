#![allow(dead_code)] // temp

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClass {
    unk1: u32,
    unk2: u32,
    ptr_descriptor: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClassDescriptor {
    unk1: u32,
    unk2: u32,
    ptr_name: u32,
    unk3: u32,
    ptr_super_class_name: u32,
    unk4: u32,
    unk5: u16,
    unk6: u16,
    unk7: u32,
    unk8: u32,
    unk9: u8,
    unk10: u8,
    unk11: u16,
    unk12: u32,
    fn_unk1: u32,
    fn_unk2: u32,
    fn_unk3: u32,
    ptr_methods: u32,
    ptr_fields: u32,
    unk13: u32,
    unk14: u32,
    unk15: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClassField {
    ptr_class: u32,
    ptr_name: u32,
    ptr_type: u32,
    unk1: u16,
    unk2: u16,
    index: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClassFields {
    count: u32,
    fields: [LgtJavaClassField; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClassMethod {
    ptr_class: u32,
    ptr_name: u32,
    ptr_type: u32,
    unk1: u16,
    unk2: u16,
    unk3: u32,
    ptr_method: u32,
    unk4: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaClassMethods {
    count: u32,
    methods: [LgtJavaClassMethod; 0],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaImportedClass {
    ptr_name: u32,
    unk1: u32,
    static_field_offset: u16,
    static_field_count: u16,
    virtual_method_offset: u16,
    virtual_method_count: u16,
    unk2: u32,
    static_method_offset: u16,
    static_method_count: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaImportedClasses {
    count: u32,
    classes: [LgtJavaImportedClass; 0],
}

#[repr(C)]
struct LgtJavaClassInstance {
    ptr_vtable: u32,
    unk1: u32,
    ptr_fields: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaPublicClass {
    ptr_name: u32,
    unk1: u16,
    unk2: u16,
    unk3: u32,
    virtual_method_offset: u16,
    virtual_method_count: u16,
    unk4: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LgtJavaPublicClasses {
    count: u32,
    classes: [LgtJavaPublicClass; 0],
}
