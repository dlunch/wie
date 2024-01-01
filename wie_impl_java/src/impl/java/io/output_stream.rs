use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    handle::JvmClassInstanceHandle,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.io.OutputStream
pub struct OutputStream {}

impl OutputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.OutputStream::<init>({:?})", &this);

        Ok(())
    }
}
