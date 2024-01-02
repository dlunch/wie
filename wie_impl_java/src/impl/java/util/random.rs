use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    handle::JvmClassInstanceHandle,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.util.Random
pub struct Random {}

impl Random {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("nextInt", "()I", Self::next_int, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.util.Random::<init>({:?})", &this);

        Ok(())
    }

    async fn next_int(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::warn!("stub java.util.Random::nextInt({:?})", &this);

        // TODO this random is not compatible with java's one.
        let random = context.system().random().next() as _;

        Ok(random)
    }
}
