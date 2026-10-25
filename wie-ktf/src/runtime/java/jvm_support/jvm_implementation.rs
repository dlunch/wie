use alloc::{boxed::Box, collections::BTreeMap, format, sync::Arc, vec::Vec};
use core::{
    mem::size_of,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use anyhow::Result as GcResult;
use jvm::{ClassDefinition, ClassInstance, ExternalRoots, Jvm, Result as JvmResult};
use jvm_class_proto::JavaClassProto;
use spin::Mutex;
use wipi_types::ktf::java::{
    JavaClassInstance as RawJavaClassInstance, JavaExceptionHandler as RawJavaExceptionHandler, JavaMethodDefinition as RawJavaMethod,
};

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;
use wie_util::read_generic;

use crate::runtime::java::{JavaSvcFunctions, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaClassDefinition, JavaClassInstance, KtfJvmThreadContext, array_class_instance::JavaArrayClassInstance};

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
}

impl JvmImplementation for KtfJvmImplementation {
    fn external_roots(&self) -> Option<Box<dyn ExternalRoots>> {
        Some(Box::new(self.clone()))
    }

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

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{element_type_name}");
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}

impl ExternalRoots for KtfJvmImplementation {
    fn roots(&self, objects: &[&dyn ClassInstance]) -> GcResult<Vec<usize>> {
        let mut snapshot = self.core.snapshot_roots()?;
        for address in snapshot.runtime_contexts {
            let context: KtfJvmThreadContext = read_generic(&self.core, address)?;
            let mut handler = context.current_java_exception_handler;
            while handler != 0 {
                let [ptr_method, ptr_this, ptr_old_handler]: [u32; 3] = read_generic(&self.core, handler)?;
                snapshot.words.push(ptr_this);
                let method: RawJavaMethod = read_generic(&self.core, ptr_method)?;
                // Frames without catches contain only the common linked header.
                if method.exception_table_count != 0 {
                    let raw: RawJavaExceptionHandler = read_generic(&self.core, handler)?;
                    snapshot.words.push(raw.ptr_exception_class_instance);
                    snapshot.words.extend_from_slice(&raw.context);
                    self.core.append_stack_words(raw.context[9], &mut snapshot.words)?;
                }
                handler = ptr_old_handler;
            }
        }
        snapshot.words.sort_unstable();
        snapshot.words.dedup();

        let mut roots = Vec::new();
        for object in objects {
            let address = if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
                instance.ptr_raw
            } else if let Some(array) = object.as_any().downcast_ref::<JavaArrayClassInstance>() {
                array.class_instance.ptr_raw
            } else {
                continue;
            };
            let raw: RawJavaClassInstance = read_generic(&self.core, address)?;
            let storage_size = if object.as_array_instance().is_some() {
                let length: u32 = read_generic(&self.core, raw.ptr_fields + 4)?;
                let class = JavaArrayClassDefinition::from_raw(raw.ptr_class, &self.core);
                u64::from(length) * class.element_size()? as u64 + 8
            } else {
                let class = JavaClassDefinition::from_raw(raw.ptr_class, &self.core);
                class.field_size()? as u64 + 4
            };
            // An array iterator can retain only an interior or one-past data pointer.
            for (base, last_offset) in [(address, size_of::<RawJavaClassInstance>() as u64 - 1), (raw.ptr_fields, storage_size)] {
                let index = snapshot.words.partition_point(|&word| word < base);
                if snapshot
                    .words
                    .get(index)
                    .is_some_and(|&word| u64::from(word) <= u64::from(base) + last_offset)
                {
                    roots.push(object.identity());
                    break;
                }
            }
        }
        Ok(roots)
    }
}
