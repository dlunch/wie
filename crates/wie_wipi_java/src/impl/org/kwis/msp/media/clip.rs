use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init)],
            fields: vec![],
        }
    }

    fn init(_: &mut JavaContext, instance: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::debug!("Clip::<init>({:#x}, {})", instance.ptr_instance, a0);

        Ok(())
    }
}
