use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    handle::{Array, JvmClassInstanceHandle},
    JavaContext, JavaMethodFlag, JavaResult,
};

// class java.io.InputStream
pub struct InputStream {}

impl InputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new_abstract("available", "()I", JavaMethodFlag::NONE),
                JavaMethodProto::new_abstract("read", "([BII)I", JavaMethodFlag::NONE),
                JavaMethodProto::new("read", "([B)I", Self::read, JavaMethodFlag::NONE),
                JavaMethodProto::new_abstract("read", "()I", JavaMethodFlag::NONE),
                JavaMethodProto::new_abstract("close", "()V", JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.InputStream::<init>({:?})", &this);

        Ok(())
    }

    async fn read(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>, b: JvmClassInstanceHandle<Array<i8>>) -> JavaResult<i32> {
        tracing::debug!("java.lang.InputStream::read({:?}, {:?})", &this, &b);

        let array_length = context.jvm().array_length(&b)? as i32;

        context
            .jvm()
            .invoke_virtual(&this, "java/io/InputStream", "read", "([BII)I", (b, 0, array_length))
            .await
    }
}
