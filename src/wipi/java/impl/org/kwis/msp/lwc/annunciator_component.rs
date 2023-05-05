use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaResult, Jvm};

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

    fn init(_: &mut dyn Jvm, _: u32) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::<init>");

        Ok(())
    }

    fn show(_: &mut dyn Jvm) -> JavaResult<()> {
        log::debug!("AnnunciatorComponent::show");

        Ok(())
    }
}
