use crate::wipi::java::{JavaBridge, JavaClassProto, JavaMethodProto, JavaResult};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init)],
        }
    }

    fn init(_: &mut dyn JavaBridge, _: u32) -> JavaResult<()> {
        log::debug!("Clip::<init>");

        Ok(())
    }
}
