use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format,
    string::{String, ToString},
    sync::Arc,
};
use core::{
    ops::{Deref, DerefMut},
    pin::Pin,
};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use spin::Mutex;

use wie_core_arm::ArmCore;
use wie_jvm_support::JvmImplementation;

use super::{JavaArrayClassDefinition, JavaClassDefinition};

#[derive(Clone)]
pub struct KtfJvmImplementation {
    core: ArmCore,
    classes: Arc<Mutex<BTreeMap<String, u32>>>,
}

impl KtfJvmImplementation {
    pub fn new(core: ArmCore) -> Self {
        Self {
            core,
            classes: Arc::new(Mutex::new(BTreeMap::new())),
        }
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
            let class_name = proto.name;
            let class = if let Some(&ptr_raw) = self.classes.lock().get(class_name) {
                JavaClassDefinition::restore(&mut self.core.clone(), jvm, ptr_raw, proto, context).unwrap()
            } else {
                JavaClassDefinition::new(&mut self.core.clone(), jvm, proto, context).await.unwrap()
            };

            self.classes.lock().insert(class_name.to_string(), class.ptr_raw);

            Ok(Box::new(class) as _)
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
