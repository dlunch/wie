use alloc::vec;

use java_class_proto::{JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msp.handset.Backlight
pub struct BackLight {}

impl BackLight {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, Default::default())],
            fields: vec![],
        }
    }

    async fn always_on(_: &mut Jvm, _: &mut WIPIJavaContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
