use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult};

pub struct Array {}

impl Array {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::trace!("Array::<init>");

        Ok(())
    }
}
