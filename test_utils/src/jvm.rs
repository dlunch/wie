use alloc::boxed::Box;

use jvm::{Jvm, Result as JvmResult};

use crate::runtime::DummyRuntime;

pub async fn test_jvm() -> JvmResult<Jvm> {
    let properties = [].into_iter().collect();
    let jvm = Jvm::new(java_runtime::get_bootstrap_class_loader(Box::new(DummyRuntime)), move || 0, properties).await?;

    Ok(jvm)
}
