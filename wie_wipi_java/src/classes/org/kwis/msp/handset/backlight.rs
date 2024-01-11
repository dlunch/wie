use alloc::vec;

use java_class_proto::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, JavaMethodFlag::NONE)],
            fields: vec![],
        }
    }

    async fn always_on(_: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
