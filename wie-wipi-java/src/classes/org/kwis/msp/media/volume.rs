use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.media.Volume
#[java_class(name = "org/kwis/msp/media/Volume", public)]
pub struct Volume;

#[java_methods]
impl Volume {
    #[method(public, native, static)]
    async fn get(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.Volume::get()");

        Ok(0)
    }

    #[method(public, native, static)]
    async fn set(_: &Jvm, _: &mut WieJvmContext, level: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Volume::set({level})");

        Ok(())
    }

    #[method(public, static)]
    async fn set_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, mute: bool) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setMute({volume_type}, {mute})");

        Ok(())
    }

    #[method(public, static)]
    async fn get_mute(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getMute({volume_type})");

        Ok(false)
    }

    #[method(public, static)]
    async fn set_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32, volume: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Volume::setDefaultVolume({volume_type}, {volume})");

        Ok(false)
    }

    #[method(public, static)]
    async fn get_default_volume(_: &Jvm, _: &mut WieJvmContext, volume_type: i32) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.Volume::getDefaultVolume({volume_type})");

        Ok(0)
    }
}
