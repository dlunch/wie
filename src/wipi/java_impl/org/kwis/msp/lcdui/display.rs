use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl};

// class org.kwis.msp.lcdui.Display
pub struct Display {}

impl Display {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "org/kwis/msp/lcdui/Display".to_owned(),
            methods: vec![
                JavaMethodImpl {
                    name: "<init>".into(),
                    signature: "()V".into(),
                    body: Box::new(Self::init),
                },
                JavaMethodImpl {
                    name: "getDisplay".into(),
                    signature: "(Ljava/lang/String;)Lorg/kwis/msp/lcdui/Display;".into(),
                    body: Box::new(Self::get_display),
                },
            ],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Display::init");

        Box::new(())
    }

    fn get_display(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Display::getDisplay");

        Box::new(())
    }
}
