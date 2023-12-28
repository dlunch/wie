use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    r#impl::org::kwis::msp::media::Clip,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class org.kwis.msp.media.Player
pub struct Player {}

impl Player {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("play", "(Lorg/kwis/msp/media/Clip;Z)Z", Self::play, JavaMethodFlag::STATIC),
                JavaMethodProto::new("stop", "(Lorg/kwis/msp/media/Clip;)Z", Self::stop, JavaMethodFlag::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn play(_: &mut dyn JavaContext, clip: JvmClassInstanceProxy<Clip>, repeat: bool) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({:?}, {})", &clip, repeat);

        Ok(false)
    }

    async fn stop(_: &mut dyn JavaContext, clip: JvmClassInstanceProxy<Clip>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({:?})", &clip,);

        Ok(false)
    }
}
