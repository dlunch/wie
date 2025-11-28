use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent;

impl AnnunciatorComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/AnnunciatorComponent",
            parent_class: Some("org/kwis/msp/lwc/ShellComponent"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init, Default::default()),
                JavaMethodProto::new("show", "()V", Self::show, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<AnnunciatorComponent>, a0: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::<init>({:?}, {})", &this, a0);

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/lwc/ShellComponent", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn show(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.lwc.AnnunciatorComponent::show()");

        Ok(())
    }
}
