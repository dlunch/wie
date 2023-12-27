use alloc::vec;
use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
    r#impl::java::io::InputStream,
    JavaContext, JavaFieldAccessFlag, JavaMethodFlag, JavaObjectProxy, JavaResult,
};

// class java.io.DataInputStream
pub struct DataInputStream {}

impl DataInputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/io/InputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/io/InputStream;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("available", "()I", Self::available, JavaMethodFlag::NONE),
                JavaMethodProto::new("read", "([BII)I", Self::read, JavaMethodFlag::NONE),
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("in", "Ljava/io/InputStream;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, r#in: JvmClassInstanceProxy<InputStream>) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.DataInputStream::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&r#in.class_instance)
        );

        context.jvm().put_field(
            &this.class_instance,
            "in",
            "Ljava/io/InputStream;",
            JavaValue::Object(Some(r#in.class_instance)),
        )?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::available({:#x})", context.instance_raw(&this.class_instance));

        let r#in = context.jvm().get_field(&this.class_instance, "in", "Ljava/io/InputStream;")?;
        let available = context
            .call_method(
                &JavaObjectProxy::new(context.instance_raw(r#in.as_object_ref().unwrap())),
                "available",
                "()I",
                &[],
            )
            .await?;

        Ok(available as _)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        b: JvmArrayClassInstanceProxy<i8>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.DataInputStream::read({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&b.class_instance),
            off,
            len
        );

        let r#in = context.jvm().get_field(&this.class_instance, "in", "Ljava/io/InputStream;")?;
        let b = context.instance_raw(&b.class_instance);
        let result = context
            .call_method(
                &JavaObjectProxy::new(context.instance_raw(r#in.as_object_ref().unwrap())),
                "read",
                "([BII)I",
                &[b, off as _, len as _],
            )
            .await?;

        Ok(result as _)
    }

    async fn close(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::close({:#x})", context.instance_raw(&this.class_instance));

        let r#in = context.jvm().get_field(&this.class_instance, "in", "Ljava/io/InputStream;")?;
        context
            .call_method(
                &JavaObjectProxy::new(context.instance_raw(r#in.as_object_ref().unwrap())),
                "close",
                "()V",
                &[],
            )
            .await?;

        Ok(())
    }
}
