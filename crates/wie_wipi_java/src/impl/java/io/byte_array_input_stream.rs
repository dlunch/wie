use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
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

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>, data: JavaObjectProxy<Array>) -> JavaResult<()> {
        log::trace!(
            "java.lang.ByteArrayInputStream::<init>({:#x}, {:#x})",
            this.ptr_instance,
            data.ptr_instance
        );

        context.put_field(&this.cast(), "buf", data.ptr_instance)?;
        context.put_field(&this.cast(), "pos", 0)?;

        Ok(())
    }

    async fn available(context: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>) -> JavaResult<u32> {
        log::trace!("java.lang.ByteArrayInputStream::available({:#x})", this.ptr_instance);

        let buf = JavaObjectProxy::new(context.get_field(&this.cast(), "buf")?);
        let pos = context.get_field(&this.cast(), "pos")?;
        let buf_length = context.array_length(&buf)?;

        Ok(buf_length - pos)
    }

    async fn read(
        context: &mut dyn JavaContext,
        this: JavaObjectProxy<ByteArrayInputStream>,
        b: JavaObjectProxy<Array>,
        off: u32,
        len: u32,
    ) -> JavaResult<u32> {
        log::trace!(
            "java.lang.ByteArrayInputStream::read({:#x}, {:#x}, {}, {})",
            this.ptr_instance,
            b.ptr_instance,
            off,
            len
        );

        let buf = JavaObjectProxy::<Array>::new(context.get_field(&this.cast(), "buf")?);
        let buf_length = context.array_length(&buf)?;
        let pos = context.get_field(&this.cast(), "pos")?;

        let available = buf_length - pos;
        let len_to_read = if len > available { available } else { len };
        if len_to_read == 0 {
            return Ok(0);
        }

        context
            .call_static_method(
                "java/lang/System",
                "arraycopy",
                "(Ljava/lang/Object;ILjava/lang/Object;II)V",
                &[buf.ptr_instance, pos, b.ptr_instance, off, len_to_read],
            )
            .await?;

        context.put_field(&this.cast(), "pos", pos + len)?;

        Ok(len)
    }

    async fn close(_: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>) -> JavaResult<()> {
        log::trace!("java.lang.ByteArrayInputStream::close({:#x})", this.ptr_instance);

        Ok(())
    }
}
