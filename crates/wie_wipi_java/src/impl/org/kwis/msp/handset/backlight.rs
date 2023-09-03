use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn always_on(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
