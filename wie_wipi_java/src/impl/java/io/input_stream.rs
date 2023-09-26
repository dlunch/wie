use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    Array, JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
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
                JavaMethodProto::new_abstract("close", "()V", JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<InputStream>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.InputStream::<init>({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn read(context: &mut dyn JavaContext, this: JavaObjectProxy<InputStream>, b: JavaObjectProxy<Array>) -> JavaResult<u32> {
        tracing::debug!("java.lang.InputStream::read({:#x}, {:#x})", this.ptr_instance, b.ptr_instance);

        let array_length = context.array_length(&b)?;

        context
            .call_method(&this.cast(), "read", "([BII)I", &[b.ptr_instance, 0, array_length])
            .await
    }
}
