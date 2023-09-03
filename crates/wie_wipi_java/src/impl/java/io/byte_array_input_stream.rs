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
            ],
            fields: vec![JavaFieldProto::new("buf", "[B", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>, data: JavaObjectProxy<Array>) -> JavaResult<()> {
        log::warn!(
            "stub java.lang.ByteArrayInputStream::<init>({:#x}, {:#x})",
            this.ptr_instance,
            data.ptr_instance
        );

        context.put_field(&this.cast(), "buf", data.ptr_instance)?;

        Ok(())
    }

    async fn available(_: &mut dyn JavaContext, this: JavaObjectProxy<ByteArrayInputStream>) -> JavaResult<u32> {
        log::warn!("stub java.lang.ByteArrayInputStream::available({:#x})", this.ptr_instance);

        Ok(0)
    }
}
