use alloc::{boxed::Box, collections::BTreeMap, format, sync::Arc};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_rust::ClassDefinitionImpl;
use spin::Mutex;

use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::JvmImplementation;
use wie_util::write_null_terminated_string_bytes;

use crate::runtime::java::{JavaSvcFunctions, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct KtfJvmImplementation {
    core: ArmCore,
    java_functions: JavaSvcFunctions,
}

impl KtfJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Self {
        let java_functions = Arc::new(Mutex::new(BTreeMap::new()));
        register_java_svc_handler(core, &java_functions).unwrap();

        Self {
            core: core.clone(),
            java_functions,
        }
    }

    pub fn java_functions(&self) -> JavaSvcFunctions {
        self.java_functions.clone()
    }

    async fn try_resolve_native_class(&self, jvm: &Jvm, class_name: &str) -> Option<JavaClassDefinition> {
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

        let core = &mut self.core.clone();
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
}

impl JvmImplementation for KtfJvmImplementation {
    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Box::pin(async move {
            Ok(Box::new(
                JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.java_functions.clone())
                    .await
                    .unwrap(),
            ) as _)
        })
    }

    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        let class = ClassDefinitionImpl::from_classfile(data)?;

        // Hybrid JARs hip a .class stub whose real implementation
        // lives in client.bin. When KtfClassLoader.fnGetClass returns an ARM-backed class, prefer it
        // so instances interoperate with KTF-native code paths.
        if let Some(native_class) = self.try_resolve_native_class(jvm, &class.name()).await {
            return Ok(Box::new(native_class));
        }

        Ok(Box::new(class))
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{element_type_name}");
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
