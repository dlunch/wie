use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

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

    fn init(_: JavaContext, _: u32) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::<init>");

        Ok(())
    }

    fn show(_: JavaContext) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::show");

        Ok(())
    }
}
