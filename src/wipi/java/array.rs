use crate::wipi::java::{JavaClassProto, JavaMethodProto, JavaResult};

use super::JavaContext;

pub struct Array {}

impl Array {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Array::<init>");

        Ok(())
    }
}
