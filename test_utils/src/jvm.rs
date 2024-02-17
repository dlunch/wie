use alloc::boxed::Box;
use core::future::ready;

use jvm::{Jvm, JvmResult};

use jvm_rust::{ClassDefinitionImpl, JvmDetailImpl};

use crate::runtime::DummyRuntime;

pub async fn test_jvm() -> JvmResult<Jvm> {
    let jvm = Jvm::new(JvmDetailImpl).await?;

    java_runtime::initialize(&jvm, |name, proto| {
        ready(Box::new(ClassDefinitionImpl::from_class_proto(name, proto, Box::new(DummyRuntime) as Box<_>)) as Box<_>)
    })
    .await?;

    Ok(jvm)
}
