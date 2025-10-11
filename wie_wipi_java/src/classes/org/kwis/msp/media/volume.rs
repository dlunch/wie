use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.media.Volume
pub struct Volume;

impl Volume {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Volume",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("set", "(I)V", Self::set, MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC),
                JavaMethodProto::new("get", "()I", Self::get, MethodAccessFlags::NATIVE | MethodAccessFlags::STATIC),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn get(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.Volume::get()");

        Ok(0)
    }

    async fn set(_: &Jvm, _: &mut WieJvmContext, level: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Volume::set({level})");

        Ok(())
    }
}
