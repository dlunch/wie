use alloc::{boxed::Box, format, string::ToString, vec, vec::Vec};

use async_trait::async_trait;

use classfile::{AttributeInfo, AttributeInfoCode, ClassInfo};
use java_class_proto::{JavaClassProto, JavaFieldProto, JavaMethodProto, MethodBody};
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, JavaError, JavaValue, Jvm, Method, Result as JvmResult};
use jvm_rust::{MethodBody as JvmRustMethodBody, MethodImpl};

use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string_bytes;

use crate::runtime::java::JavaSvcFunctions;

use super::class_definition::JavaClassDefinition;

async fn class_format_error(jvm: &Jvm, msg: &str) -> JavaError {
    jvm.exception("java/lang/ClassFormatError", msg).await
}

/// Build a KTF `JavaClassDefinition` from a Java class file. The resulting class lives in ARM
/// memory like any other KTF class, so its instances interoperate with KTF-native parent classes.
/// Bytecode methods execute via the jvm_rust interpreter.
pub async fn define_classfile_class(core: &mut ArmCore, jvm: &Jvm, data: &[u8], java_functions: JavaSvcFunctions) -> JvmResult<JavaClassDefinition> {
    let class = match ClassInfo::parse(data) {
        Some(x) => x,
        None => return Err(class_format_error(jvm, "Invalid class file").await),
    };

    let class_name_owned = class.this_class.to_string();

    // If KtfClassLoader's client.bin has a native-bound version of this class, prefer it.
    // WIPI .class files in JARs are typically stubs whose real implementations live in client.bin.
    if let Some(native_class) = try_resolve_native_class(core, jvm, &class_name_owned).await {
        return Ok(native_class);
    }

    let name: &'static str = Box::leak(class_name_owned.into_boxed_str());
    let parent_class: Option<&'static str> = class.super_class.map(|x| Box::leak(x.to_string().into_boxed_str()) as &'static str);

    let mut methods: Vec<JavaMethodProto<()>> = Vec::with_capacity(class.methods.len());
    for method_info in class.methods {
        let access_flags = method_info.access_flags;
        let name = method_info.name.to_string();
        let descriptor = method_info.descriptor.to_string();

        let body: Box<dyn MethodBody<JavaError, ()>> =
            if access_flags.contains(MethodAccessFlags::NATIVE) || access_flags.contains(MethodAccessFlags::ABSTRACT) {
                Box::new(AbstractCall {
                    name: name.clone(),
                    descriptor: descriptor.clone(),
                })
            } else {
                let code = extract_code(method_info.attributes);
                if let Some(code) = code {
                    let method_impl = MethodImpl::new(&name, &descriptor, JvmRustMethodBody::ByteCode(code), access_flags);
                    Box::new(ByteCodeBody { method_impl })
                } else {
                    Box::new(AbstractCall {
                        name: name.clone(),
                        descriptor: descriptor.clone(),
                    })
                }
            };

        methods.push(JavaMethodProto {
            name,
            descriptor,
            body,
            access_flags,
        });
    }

    let fields: Vec<JavaFieldProto> = class
        .fields
        .into_iter()
        .map(|f| JavaFieldProto::new(&f.name, &f.descriptor, f.access_flags))
        .collect();

    let proto = JavaClassProto::<()> {
        name,
        parent_class,
        interfaces: vec![],
        methods,
        fields,
        access_flags: class.access_flags,
    };

    match JavaClassDefinition::new(core, jvm, proto, Box::new(()) as Box<()>, java_functions).await {
        Ok(class) => Ok(class),
        Err(e) => Err(class_format_error(jvm, &format!("Failed to define class from classfile: {e}")).await),
    }
}

/// If `net/wie/KtfClassLoader` is initialized and its `fnGetClass` native lookup returns a class
/// for `class_name`, return that ARM-backed class. The KTF runtime keeps the real implementations
/// (including native methods) in client.bin, so prefer that over a bytecode stub when available.
async fn try_resolve_native_class(core: &mut ArmCore, jvm: &Jvm, class_name: &str) -> Option<JavaClassDefinition> {
    if !jvm.has_class("net/wie/KtfClassLoader") {
        return None;
    }

    let instance: ClassInstanceRef<()> = jvm
        .get_static_field("net/wie/KtfClassLoader", "instance", "Lnet/wie/KtfClassLoader;")
        .await
        .ok()?;
    if instance.is_null() {
        return None;
    }

    let fn_get_class: i32 = jvm.get_field(&instance, "fnGetClass", "I").await.ok()?;
    if fn_get_class == 0 {
        return None;
    }

    let ptr_name_size = (class_name.len() + 1) as u32;
    let ptr_name = Allocator::alloc(core, ptr_name_size).ok()?;
    if write_null_terminated_string_bytes(core, ptr_name, class_name.as_bytes()).is_err() {
        let _ = Allocator::free(core, ptr_name, ptr_name_size);
        return None;
    }

    let ptr_raw = core.run_function::<u32>(fn_get_class as _, &[ptr_name]).await.ok();
    let _ = Allocator::free(core, ptr_name, ptr_name_size);

    let ptr_raw = ptr_raw?;
    if ptr_raw == 0 {
        return None;
    }

    Some(JavaClassDefinition::from_raw(ptr_raw, core))
}

fn extract_code(attributes: Vec<AttributeInfo>) -> Option<AttributeInfoCode> {
    for attr in attributes {
        if let AttributeInfo::Code(code) = attr {
            return Some(code);
        }
    }
    None
}

struct ByteCodeBody {
    method_impl: MethodImpl,
}

#[async_trait]
impl MethodBody<JavaError, ()> for ByteCodeBody {
    async fn call(&self, jvm: &Jvm, _: &mut (), args: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        <MethodImpl as Method>::run(&self.method_impl, jvm, args).await
    }
}

struct AbstractCall {
    name: alloc::string::String,
    descriptor: alloc::string::String,
}

#[async_trait]
impl MethodBody<JavaError, ()> for AbstractCall {
    async fn call(&self, jvm: &Jvm, _: &mut (), _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
        Err(jvm
            .exception(
                "java/lang/AbstractMethodError",
                &format!("Abstract or native method {}{} called", self.name, self.descriptor),
            )
            .await)
    }
}
