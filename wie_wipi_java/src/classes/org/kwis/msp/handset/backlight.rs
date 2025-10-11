use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.handset.Backlight
pub struct BackLight;

impl BackLight {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/handset/BackLight",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("alwaysOn", "()V", Self::always_on, MethodAccessFlags::STATIC)],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn always_on(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.handset.Backlight::alwaysOn");

        Ok(())
    }
}
