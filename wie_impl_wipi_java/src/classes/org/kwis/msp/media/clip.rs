use alloc::vec;

use java_runtime::classes::java::lang::String;
use java_runtime_base::{Array, JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{classes::org::kwis::msp::media::PlayListener, WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
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
        _: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<Self>,
        r#type: JvmClassInstanceHandle<String>,
        resource_name: JvmClassInstanceHandle<String>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, &r#type, &resource_name);

        Ok(())
    }

    async fn init_with_data(
        _: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<Self>,
        r#type: JvmClassInstanceHandle<String>,
        data: JvmClassInstanceHandle<Array<i8>>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, r#type, &data);

        Ok(())
    }

    async fn set_volume(_: &mut Jvm, _: &mut WIPIJavaContxt, this: JvmClassInstanceHandle<Clip>, level: i32) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setVolume({:?}, {})", &this, level);

        Ok(())
    }

    async fn set_listener(
        _: &mut Jvm,
        _: &mut WIPIJavaContxt,
        this: JvmClassInstanceHandle<Self>,
        listener: JvmClassInstanceHandle<PlayListener>,
    ) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({:?}, {:?})", &this, &listener);

        Ok(())
    }
}
