#![no_std]
extern crate alloc;

mod context;
mod jvm_implementation;
mod runtime;

use alloc::{boxed::Box, format, string::String};

use java_runtime::Runtime;
use jvm::{runtime::JavaLangString, JavaError, Jvm, Result as JvmResult};

use wie_backend::System;

pub use context::{WieJavaClassProto, WieJvmContext};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::JvmRuntime;

pub struct JvmSupport;

impl JvmSupport {
    pub async fn new_jvm<T>(system: &System, jar_name: Option<&str>, protos: Box<[Box<[WieJavaClassProto]>]>, implementation: T) -> JvmResult<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let runtime = JvmRuntime::new(system.clone(), implementation.clone());

        let properties = [("file.encoding", "EUC-KR"), ("java.class.path", jar_name.unwrap_or(""))]
            .into_iter()
            .collect();
        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await?;
        let context = Box::new(WieJvmContext::new(system));

        for proto in protos.into_vec().into_iter().flat_map(|x| x.into_vec()) {
            let class = implementation.define_class_wie(&jvm, proto, context.clone()).await?;

            jvm.register_class(class, None).await?;
            // TODO add class loader
        }

        Ok(jvm)
    }

    pub async fn format_err(jvm: &Jvm, err: JavaError) -> String {
        if let JavaError::JavaException(x) = err {
            let to_string = jvm.invoke_virtual(&x, "toString", "()Ljava/lang/String;", ()).await.unwrap();

            JavaLangString::to_rust_string(jvm, &to_string).await.unwrap()
        } else {
            format!("{:?}", err)
        }
    }
}
