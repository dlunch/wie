use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    handle::{Array, JvmClassInstanceHandle},
    r#impl::java::io::InputStream,
    JavaContext, JavaFieldAccessFlag, JavaMethodFlag, JavaResult,
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
                JavaMethodProto::new("readByte", "()B", Self::read_byte, JavaMethodFlag::NONE),
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("in", "Ljava/io/InputStream;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(
        context: &mut dyn JavaContext,
        mut this: JvmClassInstanceHandle<Self>,
        r#in: JvmClassInstanceHandle<InputStream>,
    ) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::<init>({:?}, {:?})", &this, &r#in);

        context.jvm().put_field(&mut this, "in", "Ljava/io/InputStream;", r#in)?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::available({:?})", &this);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        let available: i32 = context.jvm().invoke_virtual(&r#in, "java/io/InputStream", "available", "()I", []).await?;

        Ok(available)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceHandle<Self>,
        b: JvmClassInstanceHandle<Array<i8>>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::read({:?}, {:?}, {}, {})", &this, &b, off, len);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        let result: i32 = context
            .jvm()
            .invoke_virtual(&r#in, "java/io/InputStream", "read", "([BII)I", (b, off, len))
            .await?;

        Ok(result)
    }

    async fn read_byte(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<i8> {
        tracing::debug!("java.lang.DataInputStream::readByte({:?})", &this);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        let result: i32 = context.jvm().invoke_virtual(&r#in, "java/io/InputStream", "read", "()I", []).await?;

        Ok(result as _)
    }

    async fn close(context: &mut dyn JavaContext, this: JvmClassInstanceHandle<Self>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::close({:?})", &this);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        context.jvm().invoke_virtual(&r#in, "java/io/InputStream", "close", "()V", []).await?;

        Ok(())
    }
}
