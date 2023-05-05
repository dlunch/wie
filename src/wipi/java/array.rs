use crate::wipi::java::{JavaBridge, JavaClassProto, JavaMethodProto, JavaResult};

pub struct Array {}

impl Array {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut dyn JavaBridge) -> JavaResult<()> {
        log::debug!("Array::<init>");

        Ok(())
    }
}
