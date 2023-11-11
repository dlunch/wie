use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    r#impl::java::io::InputStream,
    Array, JavaContext, JavaFieldAccessFlag, JavaMethodFlag, JavaObjectProxy, JavaResult,
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
            fields: vec![JavaFieldProto::new("in", "Ljava/io/InputStream", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<DataInputStream>, r#in: JavaObjectProxy<InputStream>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::<init>({:#x}, {:#x})", this.ptr_instance, r#in.ptr_instance);

        context.put_field(&this.cast(), "in", r#in.ptr_instance)?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JavaObjectProxy<DataInputStream>) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::available({:#x})", this.ptr_instance);

        let r#in = JavaObjectProxy::new(context.get_field(&this.cast(), "in")?);
        let available = context.call_method(&r#in, "available", "()I", &[]).await?;

        Ok(available as _)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<DataInputStream>,
        b: JavaObjectProxy<Array>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.DataInputStream::read({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            b.ptr_instance,
            off,
            len
        );

        let r#in = JavaObjectProxy::new(context.get_field(&this.cast(), "in")?);
        let result = context
            .call_method(&r#in, "read", "([BII)I", &[b.ptr_instance, off as _, len as _])
            .await?;

        Ok(result as _)
    }

    async fn close(context: &mut dyn JavaContext, this: JavaObjectProxy<DataInputStream>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::close({:#x})", this.ptr_instance);

        let r#in = JavaObjectProxy::new(context.get_field(&this.cast(), "in")?);
        context.call_method(&r#in, "close", "()V", &[]).await?;

        Ok(())
    }
}
