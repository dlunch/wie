use alloc::vec;
use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    proxy::{Array, JvmClassInstanceProxy},
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
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("in", "Ljava/io/InputStream;", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, r#in: JvmClassInstanceProxy<InputStream>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::<init>({:?}, {:?})", &this, &r#in);

        context
            .jvm()
            .put_field(&this, "in", "Ljava/io/InputStream;", JavaValue::Object(r#in.class_instance))?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::available({:?})", &this);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        let available = context
            .jvm()
            .invoke_method(r#in.as_object_ref().unwrap(), "java/io/InputStream", "available", "()I", &[])
            .await?;

        Ok(available.as_int())
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        b: JvmClassInstanceProxy<Array<i8>>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!("java.lang.DataInputStream::read({:?}, {:?}, {}, {})", &this, &b, off, len);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        let result = context
            .jvm()
            .invoke_method(
                r#in.as_object_ref().unwrap(),
                "java/io/InputStream",
                "read",
                "([BII)I",
                &[JavaValue::Object(b.class_instance), JavaValue::Int(off), JavaValue::Int(len)],
            )
            .await?;

        Ok(result.as_int())
    }

    async fn close(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<()> {
        tracing::debug!("java.lang.DataInputStream::close({:?})", &this);

        let r#in = context.jvm().get_field(&this, "in", "Ljava/io/InputStream;")?;
        context
            .jvm()
            .invoke_method(r#in.as_object_ref().unwrap(), "java/io/InputStream", "close", "()V", &[])
            .await?;

        Ok(())
    }
}
