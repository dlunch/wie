use crate::wipi::java::{JavaBridge, JavaClassProto, JavaMethodProto, JavaResult};

// class org.kwis.msp.lwc.AnnunciatorComponent
pub struct AnnunciatorComponent {}

impl AnnunciatorComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![
                JavaMethodProto::new("<init>", "(Z)V", Self::init),
                JavaMethodProto::new("show", "()V", Self::show),
            ],
        }
    }

    fn init(_: &mut dyn JavaBridge, _: u32) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::<init>");

        Ok(())
    }

    fn show(_: &mut dyn JavaBridge) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::show");

        Ok(())
    }
}
