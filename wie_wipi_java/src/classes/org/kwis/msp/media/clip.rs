use alloc::vec;

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

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
            fields: vec![JavaFieldProto::new("data", "[B", Default::default())],
        }
    }

    async fn init(
        jvm: &Jvm,
        context: &mut WIPIJavaContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        resource_name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, &r#type, &resource_name);

        let resource_name = JavaLangString::to_rust_string(jvm, &resource_name).await?;
        let resource_id = context.system().resource().id(&resource_name).unwrap();

        let resource = context.system().resource().data(resource_id).to_vec();

        let mut data_array = jvm.instantiate_array("B", resource.len()).await?;
        jvm.store_byte_array(&mut data_array, 0, cast_vec(resource)).await?;

        jvm.invoke_special(
            &this,
            "org/kwis/msp/media/Clip",
            "<init>",
            "(Ljava/lang/String;[B)V",
            (r#type, data_array),
        )
        .await?;

        Ok(())
    }

    async fn init_with_data(
        jvm: &Jvm,
        _: &mut WIPIJavaContext,
        mut this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, r#type, &data);

        jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        jvm.put_field(&mut this, "data", "[B", data).await?;

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
