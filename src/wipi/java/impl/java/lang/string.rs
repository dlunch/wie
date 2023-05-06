use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: JavaContext) -> JavaResult<()> {
        log::debug!("String::<init>");

        Ok(())
    }
}
