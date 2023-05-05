use crate::wipi::java_impl::{JavaClassImpl, JavaMethodImpl, Jvm};

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_java_impl() -> JavaClassImpl {
        JavaClassImpl {
            methods: vec![JavaMethodImpl {
                name: "<init>".into(),
                signature: "()V".into(),
                body: Box::new(Self::init),
            }],
        }
    }

    fn init(_: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        log::debug!("Exception::<init>");

        0
    }
}
