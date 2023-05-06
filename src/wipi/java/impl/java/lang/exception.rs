use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Exception::<init>");

        Ok(())
    }
}
