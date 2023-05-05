use crate::wipi::java::{JavaBridge, JavaClassProto, JavaMethodProto, JavaResult};

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn JavaBridge) -> JavaResult<()> {
        log::debug!("Exception::<init>");

        Ok(())
    }
}
