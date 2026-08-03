use alloc::{boxed::Box, collections::BTreeMap, format, string::ToString, sync::Arc};
use core::{ops::Deref, ops::DerefMut, pin::Pin};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use jvm_rust::{ClassDefinitionError, ClassDefinitionImpl};
use spin::Mutex;

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;
use wie_util::Result;

use crate::runtime::java::{JavaSvcFunctions, register_java_svc_handler};

use super::{ClassRegistry, JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct LgtJvmImplementation {
    core: ArmCore,
    functions: JavaSvcFunctions,
    registry: ClassRegistry,
}

impl LgtJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Result<Self> {
        let functions = Arc::new(Mutex::new(BTreeMap::new()));
        register_java_svc_handler(core, &functions)?;

        Ok(Self {
            core: core.clone(),
            functions,
            registry: Arc::new(Mutex::new(BTreeMap::new())),
        })
    }
}

impl JvmImplementation for LgtJvmImplementation {
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
            match JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.functions.clone(), self.registry.clone()).await {
                Ok(class) => Ok(Box::new(class) as Box<_>),
                Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
            }
        })
    }

    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        match ClassDefinitionImpl::from_classfile(data) {
            Ok(class) => Ok(Box::new(class)),
            Err(ClassDefinitionError::InvalidClassFile) => Err(jvm.exception("java/lang/ClassFormatError", "Invalid class file").await),
            Err(ClassDefinitionError::UnsupportedClassVersion(version)) => Err(jvm
                .exception(
                    "java/lang/UnsupportedClassVersionError",
                    &format!("Unsupported class file version {version}"),
                )
                .await),
            Err(ClassDefinitionError::Verification) => Err(jvm.exception("java/lang/VerifyError", "Bytecode verification failed").await),
            Err(ClassDefinitionError::UnsupportedFeature(feature)) => Err(jvm
                .exception(
                    "java/lang/UnsupportedOperationException",
                    &format!("Unsupported class file feature: {feature}"),
                )
                .await),
        }
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        match JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, element_type_name, self.registry.clone()).await {
            Ok(class) => Ok(Box::new(class)),
            Err(error) => Err(jvm.exception("net/wie/WieError", &error.to_string()).await),
        }
    }
}
