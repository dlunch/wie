use alloc::boxed::Box;

use jvm::{ClassDefinition, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_rust::ClassDefinitionImpl;

use wie_core_arm::{Allocator, ArmCore};
use wie_util::write_null_terminated_string_bytes;

use super::class_definition::JavaClassDefinition;

/// Build a `ClassDefinition` for a `.class` file loaded by a KTF app.
///
/// WIPI JARs occasionally ship a stub `.class` whose real implementation lives in `client.bin`
/// (e.g. `Clet.class` in 삼국지 영걸전). When that happens, prefer the ARM-backed class from
/// `KtfClassLoader.fnGetClass` so instances interoperate with KTF-native parent classes.
/// Otherwise fall back to the pure-Rust `jvm_rust` interpreter.
pub async fn define_classfile_class(core: &mut ArmCore, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
    let class = ClassDefinitionImpl::from_classfile(data)?;

    if let Some(native_class) = try_resolve_native_class(core, jvm, &class.name()).await {
        return Ok(Box::new(native_class));
    }

    Ok(Box::new(class))
}

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
