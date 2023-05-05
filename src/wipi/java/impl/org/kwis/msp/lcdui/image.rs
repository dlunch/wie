use crate::wipi::java::{JavaClassProto, JavaMethodProto, Jvm};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "(I)V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm, _: u32) {
        log::debug!("Image::<init>");
    }
}
