use alloc::vec;

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{
    Array, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangString},
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::media::Player;

use crate::classes::org::kwis::msp::media::PlayListener;

// class org.kwis.msp.media.Clip
pub struct Clip;

impl Clip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/Clip",
            parent_class: Some("org/kwis/msp/media/BaseClip"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;[B)V", Self::init_with_data, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init_with_data_size, Default::default()),
                JavaMethodProto::new(
                    "<init>",
                    "(Ljava/lang/String;Ljava/lang/String;)V",
                    Self::init_with_resource,
                    Default::default(),
                ),
                JavaMethodProto::new("setVolume", "(I)Z", Self::set_volume, Default::default()),
                JavaMethodProto::new(
                    "setListener",
                    "(Lorg/kwis/msp/media/PlayListener;)V",
                    Self::set_listener,
                    Default::default(),
                ),
                JavaMethodProto::new("setBuffer", "([BI)V", Self::set_buffer, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("player", "Ljavax/microedition/media/Player;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?})", &this, r#type);

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/media/BaseClip", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_resource(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        resource_name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, &r#type, &resource_name);

        let class = jvm.invoke_virtual(&r#type, "getClass", "()Ljava/lang/Class;", ()).await?;
        let resource_stream = jvm
            .invoke_virtual(
                &class,
                "getResourceAsStream",
                "(Ljava/lang/String;)Ljava/io/InputStream;",
                (resource_name.clone(),),
            )
            .await?;
        let data = JavaIoInputStream::read_until_end(jvm, &resource_stream).await?;

        let mut data_array = jvm.instantiate_array("B", data.len()).await?;
        jvm.store_array(&mut data_array, 0, cast_vec::<u8, i8>(data)).await?;

        let _: () = jvm
            .invoke_special(
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
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        data: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?}, {:?})", &this, r#type, &data);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        let length = jvm.array_length(&data).await?;

        let _: () = jvm.invoke_virtual(&this, "setBuffer", "([BI)V", (data, length as i32)).await?;

        Ok(())
    }

    async fn init_with_data_size(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({:?}, {:?}, {})", &this, r#type, size);

        let data = jvm.instantiate_array("B", size as _).await?;

        let _: () = jvm.invoke_virtual(&this, "setBuffer", "([BI)V", (data, size)).await?;

        Ok(())
    }

    async fn set_volume(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Clip>, level: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setVolume({:?}, {})", &this, level);

        Ok(())
    }

    async fn set_listener(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, listener: ClassInstanceRef<PlayListener>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({:?}, {:?})", &this, &listener);

        Ok(())
    }

    async fn set_buffer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::setBuffer({:?}, {:?}, {})", &this, &buffer, size);

        let input_stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (buffer,)).await?;
        let r#type = JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?;

        let player: ClassInstanceRef<Player> = jvm
            .invoke_static(
                "javax/microedition/media/Manager",
                "createPlayer",
                "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                (input_stream, r#type),
            )
            .await?;

        jvm.put_field(&mut this, "player", "Ljavax/microedition/media/Player;", player).await?;

        Ok(())
    }

    pub async fn player(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Player>> {
        jvm.get_field(this, "player", "Ljavax/microedition/media/Player;").await
    }
}
