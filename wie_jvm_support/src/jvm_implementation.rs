use alloc::boxed::Box;

use java_runtime::{Runtime, RuntimeClassProto};
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use jvm_rust::{ArrayClassDefinitionImpl, ClassDefinitionImpl};

use crate::{WieJavaClassProto, WieJvmContext};

#[async_trait::async_trait]
pub trait JvmImplementation: Clone {
    async fn define_class_wie(&self, jvm: &Jvm, proto: WieJavaClassProto, context: Box<WieJvmContext>) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_class_rust(&self, jvm: &Jvm, proto: RuntimeClassProto, runtime: Box<dyn Runtime>) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>>;
    async fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>>;
}

#[derive(Clone)]
pub struct RustJavaJvmImplementation;

#[async_trait::async_trait]
impl JvmImplementation for RustJavaJvmImplementation {
    async fn define_class_wie(&self, _jvm: &Jvm, proto: WieJavaClassProto, context: Box<WieJvmContext>) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, context)))
    }

    async fn define_class_rust(&self, _jvm: &Jvm, proto: RuntimeClassProto, runtime: Box<dyn Runtime>) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, runtime)))
    }

    async fn define_class_java(&self, _jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        ClassDefinitionImpl::from_classfile(data).map(|x| Box::new(x) as Box<_>)
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}
