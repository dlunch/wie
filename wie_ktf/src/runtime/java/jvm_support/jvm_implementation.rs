use alloc::{boxed::Box, format};

use java_runtime::{Runtime, RuntimeClassProto};
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::{JvmImplementation, WieJavaClassProto, WieJvmContext};

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct KtfJvmImplementation {
    core: ArmCore,
}

impl KtfJvmImplementation {
    pub fn new(core: ArmCore) -> Self {
        Self { core }
    }
}

#[async_trait::async_trait]
impl JvmImplementation for KtfJvmImplementation {
    async fn define_class_wie(&self, jvm: &Jvm, proto: WieJavaClassProto, context: Box<WieJvmContext>) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(
            JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context).await.unwrap(),
        ))
    }

    async fn define_class_rust(&self, jvm: &Jvm, proto: RuntimeClassProto, runtime: Box<dyn Runtime>) -> jvm::Result<Box<dyn ClassDefinition>> {
        Ok(Box::new(
            JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, runtime).await.unwrap(),
        ))
    }

    async fn define_class_java(&self, _jvm: &Jvm, _data: &[u8]) -> jvm::Result<Box<dyn ClassDefinition>> {
        unimplemented!()
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> jvm::Result<Box<dyn ClassDefinition>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
