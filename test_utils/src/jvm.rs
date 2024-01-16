use alloc::boxed::Box;
use core::future::ready;

use jvm::Jvm;

use jvm_rust::{ClassImpl, JvmDetailImpl};

use crate::runtime::DummyRuntime;

pub async fn test_jvm() -> anyhow::Result<Jvm> {
    let jvm = Jvm::new(JvmDetailImpl::new()).await?;

    java_runtime::initialize(&jvm, |name, proto| {
        ready(Box::new(ClassImpl::from_class_proto(name, proto, Box::new(DummyRuntime) as Box<_>)) as Box<_>)
    })
    .await?;

    Ok(jvm)
}
