use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
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

    async fn println(context: &mut dyn JavaContext, this: JavaObjectProxy<PrintStream>, str: JavaObjectProxy<String>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.PrintStream::println({:#x}, {:#x})", this.ptr_instance, str.ptr_instance);

        let rust_str = String::to_rust_string(context, &str)?;
        tracing::info!("println: {}", rust_str);

        Ok(())
    }
}
