use crate::wipi::java::{JavaClassProto, JavaMethodProto, Jvm};

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm) {
        log::debug!("Exception::<init>");
    }
}
