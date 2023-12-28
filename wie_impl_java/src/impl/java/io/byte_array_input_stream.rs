use alloc::vec;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
    JavaContext, JavaFieldAccessFlag, JavaMethodFlag, JavaResult,
};

// class java.io.ByteArrayInputStream
pub struct ByteArrayInputStream {}

impl ByteArrayInputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/io/InputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "([B)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("available", "()I", Self::available, JavaMethodFlag::NONE),
                JavaMethodProto::new("read", "([BII)I", Self::read, JavaMethodFlag::NONE),
                JavaMethodProto::new("close", "()V", Self::close, JavaMethodFlag::NONE),
            ],
            fields: vec![
                JavaFieldProto::new("buf", "[B", JavaFieldAccessFlag::NONE),
                JavaFieldProto::new("pos", "I", JavaFieldAccessFlag::NONE),
            ],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, data: JvmArrayClassInstanceProxy<i8>) -> JavaResult<()> {
        tracing::debug!("java.lang.ByteArrayInputStream::<init>({:?}, {:?})", &this, &data);

        context
            .jvm()
            .put_field(this.class_instance.as_ref().unwrap(), "buf", "[B", JavaValue::Object(data.class_instance))?;
        context
            .jvm()
            .put_field(this.class_instance.as_ref().unwrap(), "pos", "I", JavaValue::Int(0))?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!("java.lang.ByteArrayInputStream::available({:?})", &this);

        let buf = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "buf", "[B")?;
        let pos = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "pos", "I")?.as_int();
        let buf_length = context.jvm().array_length(buf.as_object_ref().unwrap())? as i32;

        Ok((buf_length - pos) as _)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        b: JvmArrayClassInstanceProxy<i8>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!("java.lang.ByteArrayInputStream::read({:?}, {:?}, {}, {})", &this, &b, off, len);

        let buf = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "buf", "[B")?;
        let buf_length = context.jvm().array_length(buf.as_object_ref().unwrap())?;
        let pos = context.jvm().get_field(this.class_instance.as_ref().unwrap(), "pos", "I")?.as_int();

        let available = (buf_length as i32 - pos) as _;
        let len_to_read = if len > available { available } else { len };
        if len_to_read == 0 {
            return Ok(0);
        }

        context
            .jvm()
            .invoke_static_method(
                "java/lang/System",
                "arraycopy",
                "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                &[
                    buf,
                    JavaValue::Int(pos as _),
                    JavaValue::Object(b.class_instance),
                    JavaValue::Int(off as _),
                    JavaValue::Int(len_to_read as _),
                ],
            )
            .await?;

        context
            .jvm()
            .put_field(this.class_instance.as_ref().unwrap(), "pos", "I", JavaValue::Int(pos + len))?;

        Ok(len)
    }

    async fn close(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<ByteArrayInputStream>) -> JavaResult<()> {
        tracing::debug!("java.lang.ByteArrayInputStream::close({:?})", &this);

        Ok(())
    }
}
