use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.BackLight
pub struct BackLight;

impl BackLight {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/BackLight",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("on", "(I)V", Self::on, MethodAccessFlags::STATIC)],
            fields: vec![],
        }
    }

    async fn on(_jvm: &Jvm, _: &mut WieJvmContext, timeout: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.BackLight::on({:?})", timeout);

        Ok(())
    }
}
