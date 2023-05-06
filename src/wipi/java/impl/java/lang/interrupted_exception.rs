use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("InterruptedException::<init>");

        Ok(())
    }
}
