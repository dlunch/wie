use std::{fmt::Display, mem::size_of};

use crate::{
    core::arm::{ArmCore, EmulatedFunctionParam},
    wipi::java::{get_java_impl, JavaMethodBody},
};

use super::Context;

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClass {
    ptr_next: u32,
    unk1: u32,
    ptr_descriptor: u32,
    unk2: u32,
    unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClassDescriptor {
    ptr_name: u32,
    unk1: u32,
    parent_class: u32,
    ptr_methods: u32,
    ptr_interfaces: u32,
    ptr_properties: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaMethod {
    fn_body: u32,
    ptr_class: u32,
    unk1: u32,
    ptr_name: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct JavaClassInstance {
    ptr_class: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WIPIJBInterface {
    unk1: u32,
    fn_unk1: u32,
    fn_java_throw: u32,
    unk3: u32,
    get_java_method: u32,
    unk4: u32,
    fn_unk4: u32,
    fn_unk5: u32,
    unk7: u32,
    unk8: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

#[derive(Clone)]
pub struct JavaMethodQualifier {
    pub tag: u8,
    pub name: String,
    pub signature: String,
}

impl JavaMethodQualifier {
    pub fn from_ptr(core: &ArmCore, ptr: u32) -> anyhow::Result<Self> {
        let tag = core.read(ptr)?;

        let value = core.read_null_terminated_string(ptr + 1)?;
        let value = value.split('+').collect::<Vec<_>>();

        Ok(JavaMethodQualifier {
            tag,
            name: value[1].into(),
            signature: value[0].into(),
        })
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        bytes.push(self.tag);
        bytes.extend_from_slice(self.signature.as_bytes());
        bytes.push(b'+');
        bytes.extend_from_slice(self.name.as_bytes());
        bytes.push(0);

        bytes
    }
}

impl EmulatedFunctionParam<JavaMethodQualifier> for JavaMethodQualifier {
    fn get(core: &mut ArmCore, pos: usize) -> JavaMethodQualifier {
        let ptr = Self::read(core, pos);

        Self::from_ptr(core, ptr).unwrap()
    }
}

impl Display for JavaMethodQualifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.signature.fmt(f)?;
        write!(f, "+")?;
        self.name.fmt(f)?;
        write!(f, "@{}", self.tag)?;

        Ok(())
    }
}

impl PartialEq for JavaMethodQualifier {
    fn eq(&self, other: &Self) -> bool {
        self.signature == other.signature && self.name == other.name
    }
}

pub fn get_wipi_jb_interface(core: &mut ArmCore, context: &Context) -> anyhow::Result<u32> {
    let interface = WIPIJBInterface {
        unk1: 0,
        fn_unk1: core.register_function(jb_unk1, context)?,
        fn_java_throw: core.register_function(java_throw, context)?,
        unk3: 0,
        get_java_method: core.register_function(get_java_method, context)?,
        unk4: 0,
        fn_unk4: core.register_function(jb_unk4, context)?,
        fn_unk5: core.register_function(jb_unk5, context)?,
        unk7: 0,
        unk8: 0,
        fn_unk2: core.register_function(jb_unk2, context)?,
        fn_unk3: core.register_function(jb_unk3, context)?,
    };

    let address = context.alloc(size_of::<WIPIJBInterface>() as u32)?;
    core.write(address, interface)?;

    Ok(address)
}

pub fn load_java_class(core: &mut ArmCore, context: &Context, ptr_target: u32, name: String) -> anyhow::Result<u32> {
    log::debug!("load_java_class({:#x}, {})", ptr_target, name);

    let r#impl = get_java_impl(&name);
    if r#impl.is_none() {
        return Ok(1);
    }
    let r#impl = r#impl.unwrap();

    let ptr_class = context.alloc(size_of::<JavaClass>() as u32)?;
    core.write(
        ptr_class,
        JavaClass {
            ptr_next: ptr_class + 4,
            unk1: 0,
            ptr_descriptor: 0,
            unk2: 0,
            unk3: 0,
        },
    )?;

    let ptr_methods = context.alloc(((r#impl.methods.len() + 1) * size_of::<u32>()) as u32)?;

    let mut cursor = ptr_methods;
    for method in r#impl.methods {
        let qualifier = (JavaMethodQualifier {
            tag: 0,
            name: method.name,
            signature: method.signature,
        })
        .as_bytes();

        let ptr_name = context.alloc(qualifier.len() as u32)?;
        core.write_raw(ptr_name, &qualifier)?;

        let ptr_method = context.alloc(size_of::<JavaMethod>() as u32)?;
        let fn_body = register_java_proxy(core, context, method.body)?;
        core.write(
            ptr_method,
            JavaMethod {
                fn_body,
                ptr_class,
                unk1: 0,
                ptr_name,
                unk2: 0,
                unk3: 0,
                unk4: 0,
            },
        )?;

        core.write(cursor, ptr_method)?;
        cursor += 4;
    }

    let ptr_name = context.alloc((r#impl.name.len() + 1) as u32)?;
    core.write_raw(ptr_name, r#impl.name.as_bytes())?;

    let ptr_descriptor = context.alloc(size_of::<JavaClassDescriptor>() as u32)?;
    core.write(
        ptr_descriptor,
        JavaClassDescriptor {
            ptr_name,
            unk1: 0,
            parent_class: 0,
            ptr_methods,
            ptr_interfaces: 0,
            ptr_properties: 0,
            unk3: 0,
            unk4: 0,
            unk5: 0,
        },
    )?;

    core.write(ptr_class + 8, ptr_descriptor)?;

    core.write(ptr_target, ptr_class)?; // we should cache ptr_class

    Ok(0)
}

pub fn instantiate_java_class(core: &mut ArmCore, context: &Context, ptr_class: u32) -> anyhow::Result<u32> {
    let class = core.read::<JavaClass>(ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Instantiate {}", class_name);

    let ptr_instance = context.alloc(size_of::<JavaClassInstance>() as u32)?;

    core.write(ptr_instance, JavaClassInstance { ptr_class })?;

    call_java_method(
        core,
        context,
        ptr_instance,
        &JavaMethodQualifier {
            tag: 72,
            name: "<init>".into(),
            signature: "()V".into(),
        },
    )?;

    Ok(ptr_instance)
}

pub fn call_java_method(core: &mut ArmCore, context: &Context, ptr_instance: u32, qualifier: &JavaMethodQualifier) -> anyhow::Result<u32> {
    let instance = core.read::<JavaClassInstance>(ptr_instance)?;
    let class = core.read::<JavaClass>(instance.ptr_class)?;
    let class_descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;
    let class_name = core.read_null_terminated_string(class_descriptor.ptr_name)?;

    log::info!("Call {}::{}", class_name, qualifier);

    let ptr_method = get_java_method(core, context, instance.ptr_class, qualifier.to_owned())?;

    let method = core.read::<JavaMethod>(ptr_method)?;

    core.run_function(method.fn_body, &[0, ptr_instance])
}

pub fn java_throw(_: &mut ArmCore, _: &Context, error: String, a1: u32) -> anyhow::Result<u32> {
    log::error!("java_throw({}, {})", error, a1);

    Ok(0)
}

fn register_java_proxy(core: &mut ArmCore, context: &Context, body: JavaMethodBody) -> anyhow::Result<u32> {
    let closure = move |_: &mut ArmCore, _: &Context| {
        body(vec![]);

        Ok::<u32, anyhow::Error>(0u32)
    };

    core.register_function(closure, context)
}

fn get_java_method(core: &mut ArmCore, _: &Context, ptr_class: u32, qualifier: JavaMethodQualifier) -> anyhow::Result<u32> {
    log::debug!("get_java_method({:#x}, {})", ptr_class, qualifier);

    let class = core.read::<JavaClass>(ptr_class)?;
    let descriptor = core.read::<JavaClassDescriptor>(class.ptr_descriptor)?;

    let mut cursor = descriptor.ptr_methods;
    loop {
        let ptr = core.read::<u32>(cursor)?;
        if ptr == 0 {
            log::error!("Can't find function {}", qualifier);

            return Ok(0);
        }

        let current_method = core.read::<JavaMethod>(ptr)?;
        let current_qualifier = JavaMethodQualifier::from_ptr(core, current_method.ptr_name)?;

        if current_qualifier == qualifier {
            log::debug!("get_java_method result {:#x}", ptr);

            return Ok(ptr);
        }

        cursor += 4;
    }
}

fn jb_unk1(core: &mut ArmCore, _: &Context, a0: u32, address: u32) -> anyhow::Result<u32> {
    // jump?
    log::debug!("jb_unk1({:#x}, {:#x})", a0, address);

    core.run_function(address, &[a0])
}

fn jb_unk2(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk2({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk3(_: &mut ArmCore, _: &Context, string: u32, a1: u32) -> anyhow::Result<u32> {
    // register string?
    log::debug!("jb_unk3({:#x}, {:#x})", string, a1);

    Ok(string)
}

fn jb_unk4(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk4({:#x}, {:#x})", a0, a1);

    Ok(0)
}

fn jb_unk5(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    log::debug!("jb_unk5({:#x}, {:#x})", a0, a1);

    Ok(0)
}

pub fn init_unk1(_: &mut ArmCore, _: &Context, a0: u32, a1: u32) -> anyhow::Result<u32> {
    // javaNew?
    log::debug!("init_unk1({:#x}, {:#x})", a0, a1);

    Ok(0)
}
