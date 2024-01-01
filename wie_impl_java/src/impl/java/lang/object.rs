use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    handle::JvmClassInstanceHandle,
};

// class java.lang.Object
pub struct Object {}

impl Object {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("getClass", "()Ljava/lang/Class;", Self::get_class, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("java.lang.Object::<init>({:?})", &this);

        Ok(())
    }

    async fn get_class(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<JvmClassInstanceHandle<Self>> {
        tracing::warn!("stub java.lang.Object::get_class({:?})", &this);

        let result = context.jvm().new_class("java/lang/Class", "()V", []).await?;

        Ok(result.into())
    }
}
