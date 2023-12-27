use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
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

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::warn!("stub java.lang.InputStream::<init>({:#x})", context.instance_raw(&this.class_instance));

        Ok(())
    }

    async fn read(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, b: JvmArrayClassInstanceProxy<i8>) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.InputStream::read({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&b.class_instance)
        );

        let array_length = context.jvm().array_length(&b.class_instance)?;

        let this = JavaObjectProxy::new(context.instance_raw(&this.class_instance));
        let b = context.instance_raw(&b.class_instance);
        Ok(context.call_method(&this, "read", "([BII)I", &[b, 0, array_length as _]).await? as _)
    }
}
