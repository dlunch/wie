use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JvmArrayClassInstanceProxy, JvmClassInstanceProxy},
    r#impl::{java::lang::String, org::kwis::msp::media::PlayListener},
};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Ljava/lang/String;)V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init_with_data, JavaMethodFlag::NONE),
                JavaMethodProto::new("setVolume", "(I)Z", Self::set_volume, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "setListener",
                    "(Lorg/kwis/msp/media/PlayListener;)V",
                    Self::set_listener,
                    JavaMethodFlag::NONE,
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(
        _: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        r#type: JvmClassInstanceProxy<String>,
        resource_name: JvmClassInstanceProxy<String>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, &r#type, &resource_name);

        Ok(())
    }

    async fn init_with_data(
        _: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        r#type: JvmClassInstanceProxy<String>,
        data: JvmArrayClassInstanceProxy<i8>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, r#type, &data);

        Ok(())
    }

    async fn set_volume(_: &mut dyn JavaContext, this: JvmClassInstanceProxy<Clip>, level: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setVolume({:?}, {})", &this, level);

        Ok(())
    }

    async fn set_listener(
        _: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        listener: JvmClassInstanceProxy<PlayListener>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({:?}, {:?})", &this, &listener);

        Ok(())
    }
}
