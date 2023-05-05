use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl, Jvm};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "(I)V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Clip::<init>");

        Box::new(())
    }
}
