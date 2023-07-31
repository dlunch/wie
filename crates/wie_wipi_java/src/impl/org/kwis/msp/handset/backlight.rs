use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodAccessFlag, JavaMethodProto, JavaResult};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, JavaMethodAccessFlag::NONE)],
            fields: vec![],
        }
    }

    async fn always_on(_: &mut dyn JavaContext) -> JavaResult<()> {
        log::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
