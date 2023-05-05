use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaResult, Jvm};

// class java.lang.String
pub struct String {}

impl String {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn Jvm) -> JavaResult<()> {
        log::debug!("String::<init>");

        Ok(())
    }
}
