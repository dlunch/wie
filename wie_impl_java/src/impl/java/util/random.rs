use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class java.util.Random
pub struct Random {}

impl Random {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<Random>) -> JavaResult<()> {
        tracing::warn!("stub java.util.Random::<init>({:#x})", this.ptr_instance);

        Ok(())
    }
}
