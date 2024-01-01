use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    handle::JvmClassInstanceHandle,
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.io.PrintStream
pub struct PrintStream {}

impl PrintStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/io/OutputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("println", "(Ljava/lang/String;)V", Self::println, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.JavaContext::<init>({:?})", &this);

        Ok(())
    }

    async fn println(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, str: JvmClassInstanceHandle<String>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.PrintStream::println({:?}, {:?})", &this, &str);

        let rust_str = String::to_rust_string(context, &str)?;
        tracing::info!("println: {}", rust_str);

        Ok(())
    }
}
