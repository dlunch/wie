use alloc::{boxed::Box, format};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

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
        Box::pin(async move { Ok(Box::new(JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context).await.unwrap()) as _) })
    }

    async fn define_class_java(&self, _jvm: &Jvm, _data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        unreachable!()
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{}", element_type_name);
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
