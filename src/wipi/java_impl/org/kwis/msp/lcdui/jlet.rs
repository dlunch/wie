use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Jlet::<init>");

        Box::new(())
    }
}
