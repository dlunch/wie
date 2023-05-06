use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.Thread
pub struct Thread {}

impl Thread {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Thread::<init>");

        Ok(())
    }
}
