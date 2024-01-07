use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{classes::org::kwis::msp::media::Clip, WieClassProto, WieContext};

// class org.kwis.msp.media.Player
pub struct Player {}

impl Player {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("play", "(Lorg/kwis/msp/media/Clip;Z)Z", Self::play, JavaMethodFlag::STATIC),
                JavaMethodProto::new("stop", "(Lorg/kwis/msp/media/Clip;)Z", Self::stop, JavaMethodFlag::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn play(_: &mut Jvm, _: &mut WieContext, clip: JvmClassInstanceHandle<Clip>, repeat: bool) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({:?}, {})", &clip, repeat);

        Ok(false)
    }

    async fn stop(_: &mut Jvm, _: &mut WieContext, clip: JvmClassInstanceHandle<Clip>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({:?})", &clip,);

        Ok(false)
    }
}
