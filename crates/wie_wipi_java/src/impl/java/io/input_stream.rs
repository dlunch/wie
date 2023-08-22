use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaFieldProto, JavaMethodProto},
    Array, JavaContext, JavaFieldAccessFlag, JavaMethodAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.io.InputStream
pub struct InputStream {}

// TODO create ByteArrayInputStream
impl InputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "([B)V", Self::init, JavaMethodAccessFlag::NONE)],
            fields: vec![JavaFieldProto::new("buf", "[B", JavaFieldAccessFlag::NONE)],
        }
    }

    async fn init(context: &mut dyn JavaContext, this: JavaObjectProxy<InputStream>, data: JavaObjectProxy<Array>) -> JavaResult<()> {
        log::warn!("stub java.lang.InputStream::<init>({:#x}, {:#x})", this.ptr_instance, data.ptr_instance);

        context.put_field(&this.cast(), "buf", data.ptr_instance)?;

        Ok(())
    }
}
