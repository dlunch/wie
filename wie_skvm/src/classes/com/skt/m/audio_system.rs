use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::skt::m::audio_clip::AudioClip;

// class com.skt.m.AudioSystem
pub struct AudioSystem;

impl AudioSystem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/AudioSystem",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "getAudioClip",
                    "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                    Self::get_audio_clip,
                    MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new("getMaxVolume", "(Ljava/lang/String;)I", Self::get_max_volume, MethodAccessFlags::STATIC),
                JavaMethodProto::new("getVolume", "(Ljava/lang/String;)I", Self::get_volume, MethodAccessFlags::STATIC),
                JavaMethodProto::new("setVolume", "(Ljava/lang/String;I)V", Self::set_volume, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn get_audio_clip(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<AudioClip>> {
        tracing::debug!("com.skt.m.AudioSystem::getAudioClip({:?})", name);

        let audio_clip = jvm.new_class("net/wie/WieAudioClip", "(Ljava/lang/String;)V", (name,)).await?;

        Ok(audio_clip.into())
    }

    async fn get_max_volume(_jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.AudioSystem::getMaxVolume({:?})", format);

        Ok(0)
    }

    async fn get_volume(_jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.skt.m.AudioSystem::getVolume({:?})", format);

        Ok(0)
    }

    async fn set_volume(_jvm: &Jvm, _context: &mut WieJvmContext, format: ClassInstanceRef<String>, level: i32) -> JvmResult<()> {
        tracing::warn!("stub com.skt.m.AudioSystem::setVolume({:?}, {})", format, level);

        Ok(())
    }
}
