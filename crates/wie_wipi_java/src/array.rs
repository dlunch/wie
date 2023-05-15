use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

pub struct Array {}

impl Array {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
            fields: vec![],
        }
    }

    fn init(_: &mut JavaContext) -> JavaResult<()> {
        log::debug!("Array::<init>");

        Ok(())
    }
}
