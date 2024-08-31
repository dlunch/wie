use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};

use jvm::{Jvm, Result as JvmResult};

use wie_backend::System;
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation, WieJavaClassProto};

use crate::TestPlatform;

// TODO macro?
pub fn run_jvm_test<T, F>(protos: Box<[Box<[WieJavaClassProto]>]>, func: T) -> JvmResult<()>
where
    T: FnOnce(Jvm) -> F + Send + 'static,
    F: Future<Output = JvmResult<()>> + Send,
{
    let mut system = System::new(Box::new(TestPlatform), "");

    let done = Arc::new(AtomicBool::new(false));
    let done_clone = done.clone();
    let system_clone = system.clone();

    system.spawn(|| async move {
        let jvm = JvmSupport::new_jvm(&system_clone, None, protos, RustJavaJvmImplementation).await?;
        func(jvm).await?;

        done_clone.store(true, Ordering::Relaxed);

        anyhow::Ok(())
    });

    loop {
        system.tick().unwrap();
        if done.load(Ordering::Relaxed) {
            break;
        }
    }

    Ok(())
}
