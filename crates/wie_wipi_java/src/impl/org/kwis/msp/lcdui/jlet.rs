use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodProto, JavaResult};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Jlet::<init>");

        Ok(())
    }
}
