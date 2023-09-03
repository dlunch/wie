use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult};

// class org.kwis.msp.lcdui.Jlet
pub struct Jlet {}

impl Jlet {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn init(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.lcdui.Jlet::<init>");

        Ok(())
    }
}
