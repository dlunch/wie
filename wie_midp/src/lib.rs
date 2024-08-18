#![no_std]
extern crate alloc;

pub mod classes;
mod context;

use core::future::Future;

use context::MIDPJavaClassProto;
pub use context::MIDPJavaContextBase;

use alloc::boxed::Box;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

pub async fn register<T, F>(jvm: &Jvm, class_creator: T) -> JvmResult<()>
where
    T: Fn(MIDPJavaClassProto) -> F,
    F: Future<Output = Box<dyn ClassDefinition>>,
{
    // superclass should come before subclass
    let protos = [classes::javax::microedition::midlet::MIDlet::as_proto()];

    for proto in protos {
        let class = class_creator(proto).await;

        jvm.register_class(class, None).await?;
    }

    Ok(())
}
