use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init, JavaMethodAccessFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, instance: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.media.Clip::<init>({:#x}, {})", instance.ptr_instance, a0);

        Ok(())
    }
}
