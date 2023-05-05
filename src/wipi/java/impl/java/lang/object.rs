use crate::wipi::java::{JavaBridge, JavaClassProto, JavaMethodProto, JavaResult};

// class java.lang.Object
pub struct Object {}

impl Object {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn JavaBridge) -> JavaResult<()> {
        log::debug!("Object::<init>");

        Ok(())
    }
}
