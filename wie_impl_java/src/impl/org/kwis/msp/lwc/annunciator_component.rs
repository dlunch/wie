use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::JavaObjectProxy,
};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("show", "()V", Self::show, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext, this: JavaObjectProxy<AnnunciatorComponent>, a0: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({:?}, {})", this.ptr_instance, a0);

        Ok(())
    }

    async fn show(_: &mut dyn JavaContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show");

        Ok(())
    }
}
