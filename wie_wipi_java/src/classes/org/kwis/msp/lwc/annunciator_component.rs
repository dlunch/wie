use alloc::vec;

use java_class_proto::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::{ClassInstanceRef, Jvm};

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("show", "()V", Self::show, JavaMethodFlag::NONE),
            ],
            fields: vec![],
        }
    }

    async fn init(_: &mut Jvm, _: &mut WIPIJavaContxt, this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({:?}, {})", &this, a0);

        Ok(())
    }

    async fn show(_: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show");

        Ok(())
    }
}
