use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl};

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "java/lang/Exception".into(),
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("Exception::<init>");

        Box::new(())
    }
}
