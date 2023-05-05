use crate::wipi::java::{JavaClassProto, JavaMethodProto, Jvm};

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm) {
        log::debug!("InterruptedException::<init>");
    }
}
