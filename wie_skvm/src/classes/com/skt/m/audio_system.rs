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
            methods: vec![JavaMethodProto::new(
                "getAudioClip",
                "(Ljava/lang/String;)Lcom/skt/m/AudioClip;",
                Self::get_audio_clip,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn get_audio_clip(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<ClassInstanceRef<AudioClip>> {
        tracing::warn!("stub com.skt.m.AudioSystem::getAudioClip({:?})", name);

        Ok(None.into())
    }
}
