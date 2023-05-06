use crate::wipi::java::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class java.lang.Object
pub struct Object {}

impl Object {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Object::<init>");

        Ok(())
    }
}
