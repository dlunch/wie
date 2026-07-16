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
                JavaMethodProto::new("setMute", "(IZ)V", Self::set_mute, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getMute", "(I)Z", Self::get_mute, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setDefaultVolume", "(II)Z", Self::set_default_volume, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getDefaultVolume", "(I)I", Self::get_default_volume, MethodAccessFlags::STATIC),
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

    async fn set_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, mute: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setMute({volume_type}, {mute})");

        Ok(())
    }

    async fn get_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getMute({volume_type})");

        Ok(false)
    }

    async fn set_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, volume: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setDefaultVolume({volume_type}, {volume})");

        Ok(false)
    }

    async fn get_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getDefaultVolume({volume_type})");

        Ok(0)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::get_protos;

    #[test]
    fn test_volume_type_stubs_return_neutral_values() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let _: () = jvm.invoke_static("org/kwis/msp/media/Volume", "setMute", "(IZ)V", (7, true)).await?;
            let muted: bool = jvm.invoke_static("org/kwis/msp/media/Volume", "getMute", "(I)Z", (7,)).await?;
            let set_default: bool = jvm
                .invoke_static("org/kwis/msp/media/Volume", "setDefaultVolume", "(II)Z", (7, 11))
                .await?;
            let default_volume: i32 = jvm.invoke_static("org/kwis/msp/media/Volume", "getDefaultVolume", "(I)I", (7,)).await?;

            assert!(!muted);
            assert!(!set_default);
            assert_eq!(default_volume, 0);

            Ok(())
        })
    }
}
