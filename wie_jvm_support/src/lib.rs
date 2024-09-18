#![no_std]
extern crate alloc;

mod context;
mod jvm_implementation;
mod runtime;

use alloc::{boxed::Box, format};

use java_runtime::{Runtime, RT_RUSTJAR};
use jvm::{runtime::JavaLangString, JavaError, Jvm};

use wie_backend::System;
use wie_util::{Result, WieError};

pub use context::{WieJavaClassProto, WieJvmContext};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::JvmRuntime;

pub static WIE_RUSTJAR: &str = "wie.rustjar";

pub struct JvmSupport;

impl JvmSupport {
    pub async fn new_jvm<T>(system: &System, jar_name: Option<&str>, protos: Box<[Box<[WieJavaClassProto]>]>, implementation: T) -> Result<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let runtime = JvmRuntime::new(system.clone(), implementation.clone(), protos);

        let class_path = if let Some(x) = jar_name {
            format!("{}:{}:{}", RT_RUSTJAR, WIE_RUSTJAR, x)
        } else {
            format!("{}:{}", RT_RUSTJAR, WIE_RUSTJAR,)
        };

        let properties = [("file.encoding", "EUC-KR"), ("java.class.path", &class_path)].into_iter().collect();
        let jvm = Jvm::new(
            java_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
        )
        .await
        .map_err(|x| WieError::FatalError(format!("Failed to create JVM: {}", x)))?;

        Ok(jvm)
    }

    pub async fn to_wie_err(jvm: &Jvm, err: JavaError) -> WieError {
        match err {
            JavaError::JavaException(x) => {
                let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await.unwrap();
                let print_writer = jvm
                    .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                    .await
                    .unwrap();

                let _: () = jvm
                    .invoke_virtual(&x, "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                    .await
                    .unwrap();

                let trace = jvm.invoke_virtual(&string_writer, "toString", "()Ljava/lang/String;", []).await.unwrap();

                WieError::FatalError(format!("\n{}", JavaLangString::to_rust_string(jvm, &trace).await.unwrap()))
            }
            JavaError::FatalError(x) => WieError::FatalError(x),
        }
    }
}
