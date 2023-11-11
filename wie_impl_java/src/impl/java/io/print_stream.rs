use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class java.io.PrintStream
pub struct PrintStream {}

impl PrintStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/io/OutputStream"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<PrintStream>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.PrintStream::<init>({:#x})", this.ptr_instance);

        Ok(())
    }
}
