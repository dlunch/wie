use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl};

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            name: "java/lang/InterruptedException".into(),
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("InterruptedException::<init>");

        Box::new(())
    }
}
