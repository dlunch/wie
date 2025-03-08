use alloc::boxed::Box;
use core::{
    future,
    ops::{Deref, DerefMut},
    pin::Pin,
};

use java_class_proto::JavaClassProto;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};
use jvm_rust::{ArrayClassDefinitionImpl, ClassDefinitionImpl};

pub trait JvmImplementation: Clone {
    #[allow(clippy::type_complexity)]
    fn define_class_rust<'a, C, Context>(
        &'a self,
        jvm: &'a Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send + 'a>>
    // XXX we get one type is more general error if we use impl Future
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static;
    fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> impl Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send;
    fn define_array_class(&self, jvm: &Jvm, element_type_name: &str) -> impl Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send;
}

#[derive(Clone)]
pub struct RustJavaJvmImplementation;

impl JvmImplementation for RustJavaJvmImplementation {
    fn define_class_rust<C, Context>(
        &self,
        _jvm: &Jvm,
        proto: JavaClassProto<C>,
        context: Context,
    ) -> Pin<Box<dyn Future<Output = JvmResult<Box<dyn ClassDefinition>>> + Send>>
    where
        C: ?Sized + 'static + Send,
        Context: Sync + Send + DerefMut + Deref<Target = C> + Clone + 'static,
    {
        Box::pin(future::ready(Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, context)) as _)))
    }

    async fn define_class_java(&self, _jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        ClassDefinitionImpl::from_classfile(data).map(|x| Box::new(x) as Box<_>)
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}
