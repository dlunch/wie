use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "java/lang/String".into(),
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("String::init");

        Box::new(())
    }
}
