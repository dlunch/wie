use alloc::{
    boxed::Box,
    collections::BTreeMap,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::{mem::size_of, ops::Deref, ops::DerefMut, pin::Pin};

use anyhow::Result as GcResult;
use jvm::{ClassDefinition, ClassInstance, ExternalRoots, JavaType, Jvm, Result as JvmResult};
use jvm_class_proto::JavaClassProto;
use spin::Mutex;
use wipi_types::lgt::java::LgtJavaClassInstance as RawJavaClassInstance;

use wie_core_arm::ArmCore;
use wie_jvm_support::{JvmImplementation, native::array_element_size};
use wie_util::{Result, read_generic, read_null_terminated_string_bytes};

use crate::runtime::java::{JavaSvcFunctions, exception, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaArrayClassInstance, JavaClassDefinition, JavaClassInstance};

#[derive(Clone)]
pub struct LgtJvmImplementation {
    core: ArmCore,
    functions: JavaSvcFunctions,
}

impl LgtJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Result<Self> {
        let functions = Arc::new(Mutex::new(BTreeMap::new()));
        register_java_svc_handler(core, &functions)?;

        Ok(Self {
            core: core.clone(),
            functions,
        })
    }
}

impl JvmImplementation for LgtJvmImplementation {
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
            match JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.functions.clone()).await {
                Ok(class) => Ok(Box::new(class) as Box<_>),
                Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
            }
        })
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        match JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, element_type_name, self.functions.clone()).await {
            Ok(class) => Ok(Box::new(class)),
            Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
        }
    }
}

impl ExternalRoots for LgtJvmImplementation {
    fn roots(&self, objects: &[&dyn ClassInstance]) -> GcResult<Vec<usize>> {
        let mut snapshot = self.core.snapshot_roots()?;
        for context in snapshot.runtime_contexts {
            exception::root_words(&self.core, context, &mut snapshot.words)?;
        }
        snapshot.words.sort_unstable();
        snapshot.words.dedup();

        let mut roots = Vec::new();
        for object in objects {
            let (instance, storage_size) = if let Some(instance) = object.as_any().downcast_ref::<JavaClassInstance>() {
                (instance, instance.storage_size()?.max(4) as u64)
            } else if let Some(array) = object.as_any().downcast_ref::<JavaArrayClassInstance>() {
                let instance = &array.class_instance;
                let descriptor = instance.class()?.descriptor()?;
                let name = String::from_utf8(read_null_terminated_string_bytes(&self.core, descriptor.ptr_name)?)?;
                let length: u32 = read_generic(&self.core, instance.ptr_fields()?)?;
                (instance, 8 + u64::from(length) * array_element_size(&JavaType::parse(&name[1..])) as u64)
            } else {
                continue;
            };
            let header = instance.ptr_raw;
            let storage = instance.ptr_fields()?;
            let ranges = [(header, size_of::<RawJavaClassInstance>() as u64), (storage, storage_size)];
            if ranges.into_iter().any(|(start, size)| {
                let index = snapshot.words.partition_point(|word| *word < start);
                snapshot.words.get(index).is_some_and(|word| u64::from(*word) <= u64::from(start) + size)
            }) {
                roots.push(object.identity());
            }
        }
        Ok(roots)
    }
}
