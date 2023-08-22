use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    JavaContext, JavaMethodAccessFlag, JavaObjectProxy, JavaResult,
};

// class java.io.InputStream
pub struct InputStream {}

impl InputStream {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<InputStream>) -> JavaResult<()> {
        log::warn!("stub java.lang.InputStream::<init>({:#x})", this.ptr_instance);

        Ok(())
    }
}
