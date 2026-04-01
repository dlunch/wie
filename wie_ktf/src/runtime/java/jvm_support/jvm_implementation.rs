use alloc::{boxed::Box, collections::BTreeMap, format, sync::Arc};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use spin::Mutex;

use wie_core_arm::{ArmCore, SvcHandle};
use wie_jvm_support::JvmImplementation;

use crate::runtime::java::{JavaSvcFunctions, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct KtfJvmImplementation {
    core: ArmCore,
    java_handle: SvcHandle,
    java_functions: JavaSvcFunctions,
}

impl KtfJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Self {
        let java_functions = Arc::new(Mutex::new(BTreeMap::new()));
        let java_handle = register_java_svc_handler(core, &java_functions).unwrap();

        Self {
            core: core.clone(),
            java_handle,
            java_functions,
        }
    }

    pub fn java_handle(&self) -> SvcHandle {
        self.java_handle
    }

    pub fn java_functions(&self) -> JavaSvcFunctions {
        self.java_functions.clone()
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
                JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context, self.java_handle, self.java_functions.clone())
                    .await
                    .unwrap(),
            ) as _)
        })
    }

    async fn define_class_java(&self, _jvm: &Jvm, _data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        unreachable!()
    }

    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        let class_name = format!("[{element_type_name}");
        let class = JavaArrayClassDefinition::new(&mut self.core.clone(), jvm, &class_name).await.unwrap();

        Ok(Box::new(class) as Box<_>)
    }
}
