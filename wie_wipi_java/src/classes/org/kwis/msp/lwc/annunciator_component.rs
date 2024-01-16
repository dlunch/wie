use alloc::vec;

use java_class_proto::{JavaMethodProto, JavaResult};
use jvm::{ClassInstanceRef, Jvm};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, Default::default()),
                JavaMethodProto::new("show", "()V", Self::show, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({:?}, {})", &this, a0);

        Ok(())
    }

    async fn show(_: &Jvm, _: &mut WIPIJavaContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show");

        Ok(())
    }
}
