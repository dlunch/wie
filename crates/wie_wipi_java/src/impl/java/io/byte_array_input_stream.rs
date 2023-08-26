use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    Array, JavaContext, JavaFieldAccessFlag, JavaMethodAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.io.ByteArrayInputStream
pub struct ByteArrayInputStream {}

impl ByteArrayInputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "([B)V", Self::init, JavaMethodAccessFlag::NONE)],
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
}
