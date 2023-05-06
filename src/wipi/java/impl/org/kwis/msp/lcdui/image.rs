use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext, _: u32) -> JavaResult<()> {
        log::debug!("Image::<init>");

        Ok(())
    }
}
