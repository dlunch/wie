#![no_std]
extern crate alloc;

mod context;
mod jvm_implementation;
pub mod native;
mod runtime;

use alloc::{boxed::Box, format, string::ToString};

use jvm::{
    ClassInstance, JavaError, JavaValue, Jvm, Result as JvmResult,
    runtime::{JavaLangClassLoader, JavaLangString},
};
use rustjava_runtime::Runtime;

use wie_backend::System;
use wie_util::{Result, WieError};

pub use context::{WieJavaClassProto, WieJvmContext};
pub use jvm_implementation::{JvmImplementation, RustJavaJvmImplementation};
use runtime::JvmRuntime;

pub static WIE_RUSTJAR: &str = "wie.rustjar";

pub struct JvmSupport;

impl JvmSupport {
    pub async fn new_jvm<T>(
        system: &System,
        jar_name: Option<&str>,
        protos: Box<[Box<[WieJavaClassProto]>]>,
        properties: &[(&str, &str)],
        implementation: T,
    ) -> Result<Jvm>
    where
        T: JvmImplementation + Sync + Send + 'static,
    {
        let external_roots = implementation.external_roots();
        let runtime = JvmRuntime::new(system.clone(), implementation, protos);

        let class_path = if let Some(x) = jar_name {
            if cfg!(windows) {
                format!("{WIE_RUSTJAR};{x}")
            } else {
                format!("{WIE_RUSTJAR}:{x}")
            }
        } else {
            WIE_RUSTJAR.to_string()
        };

        let properties = [
            ("file.encoding", "EUC-KR"),
            ("java.class.path", &class_path),
            //("rustjava.disable_explicit_gc", "true"),
        ]
        .iter()
        .chain(properties.iter())
        .copied()
        .collect();
        let jvm = Jvm::new(
            rustjava_runtime::get_bootstrap_class_loader(Box::new(runtime.clone())),
            move || runtime.current_task_id(),
            properties,
            external_roots,
        )
        .await
        .map_err(|x| WieError::FatalError(format!("Failed to create JVM: {x}")))?;

        Ok(jvm)
    }

    // Platform APIs name app resources from the archive root, optionally with a leading '/'; class loader names carry none.
    pub async fn open_resource(jvm: &Jvm, name: &str) -> JvmResult<Option<Box<dyn ClassInstance>>> {
        let class_loader = JavaLangClassLoader::get_system_class_loader(jvm).await?;
        JavaLangClassLoader::get_resource_as_stream(jvm, &class_loader, name.trim_start_matches('/')).await
    }

    pub async fn to_wie_err(jvm: &Jvm, err: JavaError) -> WieError {
        match err {
            JavaError::JavaException(x) => {
                let _frame = jvm.push_native_frame(&[JavaValue::Object(Some(x.clone()))]);
                let string_writer = jvm.new_class("java/io/StringWriter", "()V", ()).await.unwrap();
                let print_writer = jvm
                    .new_class("java/io/PrintWriter", "(Ljava/io/Writer;)V", (string_writer.clone(),))
                    .await
                    .unwrap();

                let _: () = jvm
                    .invoke_virtual(&x, "java/lang/Throwable", "printStackTrace", "(Ljava/io/PrintWriter;)V", (print_writer,))
                    .await
                    .unwrap();

                let trace: Box<dyn ClassInstance> = jvm
                    .invoke_virtual(&string_writer, "java/io/StringWriter", "toString", "()Ljava/lang/String;", [])
                    .await
                    .unwrap();

                WieError::FatalError(format!("\n{}", JavaLangString::to_rust_string(jvm, &trace).await.unwrap()))
            }
        }
    }
}
