use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
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
            methods: vec![JavaMethodProto::new(
                "println",
                "(Ljava/lang/String;)V",
                Self::println,
                JavaMethodFlag::NONE,
            )],
            fields: vec![],
        }
    }

    async fn println(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, str: JvmClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.PrintStream::println({:?}, {:?})", &this, &str);

        let rust_str = String::to_rust_string(context, &str.class_instance)?;
        tracing::info!("println: {}", rust_str);

        Ok(())
    }
}
