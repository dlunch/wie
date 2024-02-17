use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, JvmResult};

use crate::{
    classes::org::kwis::msp::media::PlayListener,
    context::{WIPIJavaClassProto, WIPIJavaContext},
};

// class org.kwis.msp.media.Clip
pub struct Clip {}

impl Clip {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init_with_data, Default::default()),
                JavaMethodProto::new("setVolume", "(I)Z", Self::set_volume, Default::default()),
                JavaMethodProto::new(
                    "setListener",
                    "(Lorg/kwis/msp/media/PlayListener;)V",
                    Self::set_listener,
                    Default::default(),
                ),
            ],
            fields: vec![],
        }
    }

    async fn init(
        _: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        resource_name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, &r#type, &resource_name);

        Ok(())
    }

    async fn init_with_data(
        _: &Jvm,
        _: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, r#type, &data);

        Ok(())
    }

    async fn set_volume(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Clip>, level: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setVolume({:?}, {})", &this, level);

        Ok(())
    }

    async fn set_listener(_: &Jvm, _: &mut WIPIJavaContext, this: ClassInstanceRef<Self>, listener: ClassInstanceRef<PlayListener>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({:?}, {:?})", &this, &listener);

        Ok(())
    }
}
