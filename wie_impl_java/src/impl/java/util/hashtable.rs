use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.util.Hashtable
pub struct Hashtable {}

impl Hashtable {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.util.Hashtable::<init>({:?})", &this);

        Ok(())
    }
}
