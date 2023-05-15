use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init),
                JavaMethodProto::new("show", "()V", Self::show),
            ],
            fields: vec![],
        }
    }

    fn init(_: &mut JavaContext, instance: JavaObjectProxy, a0: u32) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::<init>({:#x}, {})", instance.ptr_instance, a0);

        Ok(())
    }

    fn show(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::show");

        Ok(())
    }
}
