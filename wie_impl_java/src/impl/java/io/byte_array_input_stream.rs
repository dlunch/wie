use alloc::vec;

use jvm::JavaValue;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    Array, JavaContext, JavaFieldAccessFlag, JavaMethodFlag, JavaObjectProxy, JavaResult,
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

    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, data: JvmClassInstanceProxy<Array>) -> JavaResult<()> {
        tracing::debug!(
            "java.lang.ByteArrayInputStream::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&data.class_instance)
        );

        context.jvm().put_field(
            &this.class_instance,
            "buf",
            "Ljava/lang/Object;",
            JavaValue::Object(Some(data.class_instance)),
        )?;
        context.jvm().put_field(&this.class_instance, "pos", "I", JavaValue::Integer(0))?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.ByteArrayInputStream::available({:#x})",
            context.instance_raw(&this.class_instance)
        );

        let buf = context.jvm().get_field(&this.class_instance, "buf", "[B")?;
        let pos = context.jvm().get_field(&this.class_instance, "pos", "I")?.as_integer();
        let buf_length = context.array_length(&JavaObjectProxy::new(context.instance_raw(buf.as_object().unwrap())))? as i32;

        Ok((buf_length - pos) as _)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        b: JavaObjectProxy<Array>,
        off: i32,
        len: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "java.lang.ByteArrayInputStream::read({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            b.ptr_instance,
            off,
            len
        );

        let buf = context.jvm().get_field(&this.class_instance, "buf", "[B")?;
        let buf_length = context.array_length(&JavaObjectProxy::new(context.instance_raw(buf.as_object().unwrap())))?;
        let pos = context.jvm().get_field(&this.class_instance, "pos", "I")?.as_integer();

        let available = (buf_length as i32 - pos) as _;
        let len_to_read = if len > available { available } else { len };
        if len_to_read == 0 {
            return Ok(0);
        }

        context
            .call_static_method(
                "java/lang/System",
                "arraycopy",
                "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                &[
                    context.instance_raw(buf.as_object().unwrap()),
                    pos as _,
                    b.ptr_instance,
                    off as _,
                    len_to_read as _,
                ],
            )
            .await?;

        context.jvm().put_field(&this.class_instance, "pos", "I", JavaValue::Integer(pos + len))?;

        Ok(len)
    }

    async fn close(_: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>) -> JavaResult<()> {
        tracing::debug!("java.lang.ByteArrayInputStream::close({:#x})", this.ptr_instance);

        Ok(())
    }
}
