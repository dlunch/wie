use std::any::Any;

use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl, Jvm};

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<Box<dyn Any>>) -> Box<dyn Any> {
        log::debug!("InterruptedException::<init>");

        Box::new(())
    }
}
