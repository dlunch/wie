use alloc::boxed::Box;
use core::time::Duration;

use java_runtime::{File, FileStat, IOError, Runtime, RuntimeClassProto, SpawnCallback};
use jvm::{ClassDefinition, Jvm};
use jvm_rust::{ArrayClassDefinitionImpl, ClassDefinitionImpl};

#[derive(Clone)]
pub struct DummyRuntime;

#[async_trait::async_trait]
impl Runtime for DummyRuntime {
    async fn sleep(&self, _duration: Duration) {
        todo!()
    }

    async fn r#yield(&self) {
        todo!()
    }

    fn spawn(&self, _callback: Box<dyn SpawnCallback>) {
        todo!()
    }

    fn now(&self) -> u64 {
        todo!()
    }

    fn current_task_id(&self) -> u64 {
        0 // TODO
    }

    fn stdin(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stderr(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    async fn open(&self, _path: &str) -> Result<Box<dyn File>, IOError> {
        todo!()
    }

    async fn stat(&self, _path: &str) -> Result<FileStat, IOError> {
        todo!()
    }

    async fn define_class_rust(&self, _jvm: &Jvm, proto: RuntimeClassProto) -> jvm::Result<Box<dyn ClassDefinition>> {
        Ok(Box::new(ClassDefinitionImpl::from_class_proto(proto, Box::new(self.clone()) as Box<_>)))
    }

    async fn define_class_java(&self, _jvm: &Jvm, data: &[u8]) -> jvm::Result<Box<dyn ClassDefinition>> {
        ClassDefinitionImpl::from_classfile(data).map(|x| Box::new(x) as Box<_>)
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> jvm::Result<Box<dyn ClassDefinition>> {
        Ok(Box::new(ArrayClassDefinitionImpl::new(element_type_name)))
    }
}
