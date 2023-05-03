use std::any::Any;

use crate::wipi::java::{JavaClassImpl, JavaMethodImpl};

// class org.kwis.msp.lcdui.Card
pub struct Card {}

impl Card {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "org/kwis/msp/lcdui/Card".into(),
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Card::init");

        Box::new(())
    }
}
