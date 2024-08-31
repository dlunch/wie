use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::media::Clip;

// class org.kwis.msp.media.Player
pub struct Player {}

impl Player {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Player",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("play", "(Lorg/kwis/msp/media/Clip;Z)Z", Self::play, MethodAccessFlags::STATIC),
                JavaMethodProto::new("stop", "(Lorg/kwis/msp/media/Clip;)Z", Self::stop, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn play(jvm: &Jvm, context: &mut WieJvmContext, clip: ClassInstanceRef<Clip>, repeat: bool) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Player::play({:?}, {})", &clip, repeat);

        let clip_data = Clip::data(jvm, clip).await?;

        let audio_handle = context.system().audio().load_smaf(&clip_data).unwrap();

        context.system().audio().play(audio_handle).unwrap();

        Ok(false)
    }

    async fn stop(_: &Jvm, _: &mut WieJvmContext, clip: ClassInstanceRef<Clip>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({:?})", &clip,);

        Ok(false)
    }
}
