use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult};

pub struct Array {}

impl Array {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodAccessFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::trace!("Array::<init>");

        Ok(())
    }
}
