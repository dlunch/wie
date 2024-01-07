use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{JavaClassProto, JavaContextArg};

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

    async fn always_on(_: &mut Jvm, _: &mut JavaContextArg) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
