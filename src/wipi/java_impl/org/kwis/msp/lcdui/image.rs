use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl, Jvm};

// class org.kwis.msp.lcdui.Image
pub struct Image {}

impl Image {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "(I)V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Image::<init>");

        0
    }
}
