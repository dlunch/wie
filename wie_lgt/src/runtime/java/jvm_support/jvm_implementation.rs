use alloc::{boxed::Box, collections::BTreeMap, string::ToString, sync::Arc};
use core::{ops::Deref, ops::DerefMut, pin::Pin};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use spin::Mutex;

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;
use wie_util::Result;

use crate::runtime::java::{JavaSvcFunctions, exception, register_java_svc_handler};

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct LgtJvmImplementation {
    core: ArmCore,
    functions: JavaSvcFunctions,
}

impl LgtJvmImplementation {
    pub fn new(core: &mut ArmCore) -> Result<Self> {
        let functions = Arc::new(Mutex::new(BTreeMap::new()));
        exception::init(core)?;
        register_java_svc_handler(core, &functions)?;

        Ok(Self {
            core: core.clone(),
            functions,
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
