use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, JavaMethodAccessFlag::NONE),
                JavaMethodProto::new("show", "()V", Self::show, JavaMethodAccessFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({:#x}, {})", this.ptr_instance, a0);

        Ok(())
    }

    async fn show(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show");

        Ok(())
    }
}
