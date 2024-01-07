use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{WieClassProto, WieContext};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn always_on(_: &mut Jvm, _: &mut WieContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
