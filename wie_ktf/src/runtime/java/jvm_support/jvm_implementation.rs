use alloc::{boxed::Box, format};
use core::ops::{Deref, DerefMut};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;

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
    async fn define_class_rust<C, Context>(&self, jvm: &Jvm, proto: JavaClassProto<C>, context: Context) -> JvmResult<Box<dyn ClassDefinition>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Ok(Box::new(
            JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context).await.unwrap(),
        ))
    }

    async fn define_class_java(&self, _jvm: &Jvm, _data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        unimplemented!()
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
