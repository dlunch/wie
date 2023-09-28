use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    r#impl::org::kwis::msp::media::Clip,
    JavaContext, JavaMethodFlag, JavaObjectProxy, JavaResult,
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

    async fn play(_: &mut dyn JavaContext, clip: JavaObjectProxy<Clip>, repeat: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Player::play({:#x}, {})", clip.ptr_instance, repeat);

        Ok(())
    }

    async fn stop(_: &mut dyn JavaContext, clip: JavaObjectProxy<Clip>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Player::stop({:#x})", clip.ptr_instance,);

        Ok(())
    }
}
