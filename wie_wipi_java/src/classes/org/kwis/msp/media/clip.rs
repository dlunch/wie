use alloc::vec;

use bytemuck::cast_vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaIoInputStream};

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
                JavaMethodProto::new("getType", "()Ljava/lang/String;", Self::get_type, Default::default()),
                JavaMethodProto::new("setPosition", "(I)Z", Self::set_position, Default::default()),
                JavaMethodProto::new("getPosition", "()I", Self::get_position, Default::default()),
                JavaMethodProto::new("setStopTime", "(I)Z", Self::set_stop_time, Default::default()),
                JavaMethodProto::new("getStopTime", "()I", Self::get_stop_time, Default::default()),
                JavaMethodProto::new("getVolume", "()I", Self::get_volume, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("position", "I", Default::default()),
                JavaFieldProto::new("stopTime", "I", Default::default()),
                JavaFieldProto::new("type", "Ljava/lang/String;", Default::default()),
                JavaFieldProto::new("volume", "I", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/media/BaseClip", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "type", "Ljava/lang/String;", r#type).await?;

        Ok(())
    }

    async fn init_with_resource(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        resource_name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?}, {resource_name:?})");

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
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?}, {data:?})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/media/Clip", "<init>", "(Ljava/lang/String;)V", (r#type,))
            .await?;
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
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?}, {size})");

        let data = jvm.instantiate_array("B", size as _).await?;

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/media/Clip", "<init>", "(Ljava/lang/String;[B)V", (r#type, data))
            .await?;

        Ok(())
    }

    async fn set_volume(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Clip>, level: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setVolume({this:?}, {level})");

        if !(0..=100).contains(&level) {
            return Ok(false);
        }

        jvm.put_field(&mut this, "volume", "I", level).await?;

        Ok(true)
    }

    async fn set_listener(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, listener: ClassInstanceRef<PlayListener>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn set_buffer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::setBuffer({this:?}, {buffer:?}, {size})");

        let _: i32 = jvm.invoke_virtual(&this, "putData", "([BII)I", (buffer, 0, size)).await?;

        Ok(())
    }

    async fn get_type(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.media.Clip::getType({this:?})");

        jvm.get_field(&this, "type", "Ljava/lang/String;").await
    }

    async fn set_position(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, milliseconds: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setPosition({this:?}, {milliseconds})");

        jvm.put_field(&mut this, "position", "I", milliseconds).await?;

        Ok(true)
    }

    async fn get_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getPosition({this:?})");

        jvm.get_field(&this, "position", "I").await
    }

    async fn set_stop_time(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, milliseconds: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setStopTime({this:?}, {milliseconds})");

        jvm.put_field(&mut this, "stopTime", "I", milliseconds).await?;

        Ok(true)
    }

    async fn get_stop_time(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getStopTime({this:?})");

        jvm.get_field(&this, "stopTime", "I").await
    }

    async fn get_volume(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getVolume({this:?})");

        jvm.get_field(&this, "volume", "I").await
    }

    pub async fn player(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Player>> {
        jvm.get_field(this, "player", "Ljavax/microedition/media/Player;").await
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::media::Clip, get_protos};

    #[test]
    fn test_position_and_stop_time_round_trip() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type = JavaLangString::from_rust_string(&jvm, "audio/test").await?;
            let clip: ClassInstanceRef<Clip> = jvm.new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (r#type,)).await?.into();

            let initial_position: i32 = jvm.invoke_virtual(&clip, "getPosition", "()I", ()).await?;
            let initial_stop_time: i32 = jvm.invoke_virtual(&clip, "getStopTime", "()I", ()).await?;
            let position_set: bool = jvm.invoke_virtual(&clip, "setPosition", "(I)Z", (-17,)).await?;
            let stop_time_set: bool = jvm.invoke_virtual(&clip, "setStopTime", "(I)Z", (i32::MAX,)).await?;
            let position: i32 = jvm.invoke_virtual(&clip, "getPosition", "()I", ()).await?;
            let stop_time: i32 = jvm.invoke_virtual(&clip, "getStopTime", "()I", ()).await?;

            assert_eq!(initial_position, 0);
            assert_eq!(initial_stop_time, 0);
            assert!(position_set);
            assert!(stop_time_set);
            assert_eq!(position, -17);
            assert_eq!(stop_time, i32::MAX);

            Ok(())
        })
    }

    #[test]
    fn test_string_and_data_constructor_type_round_trip() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let string_type = JavaLangString::from_rust_string(&jvm, "audio/string").await?;
            let string_clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (string_type,))
                .await?
                .into();

            let data_type = JavaLangString::from_rust_string(&jvm, "audio/data").await?;
            let mut data = jvm.instantiate_array("B", 3).await?;
            jvm.store_array(&mut data, 0, [1i8, 2, 3]).await?;
            let data_clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;[B)V", (data_type, data))
                .await?
                .into();

            let returned_string_type: ClassInstanceRef<String> = jvm.invoke_virtual(&string_clip, "getType", "()Ljava/lang/String;", ()).await?;
            let returned_data_type: ClassInstanceRef<String> = jvm.invoke_virtual(&data_clip, "getType", "()Ljava/lang/String;", ()).await?;

            assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_string_type).await?, "audio/string");
            assert_eq!(JavaLangString::to_rust_string(&jvm, &returned_data_type).await?, "audio/data");

            Ok(())
        })
    }

    #[test]
    fn test_volume_range_round_trip() -> Result<()> {
        run_jvm_test(Box::new([wie_midp::get_protos().into(), get_protos().into()]), |jvm| async move {
            let r#type = JavaLangString::from_rust_string(&jvm, "audio/test").await?;
            let clip: ClassInstanceRef<Clip> = jvm.new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (r#type,)).await?.into();
            let second_type = JavaLangString::from_rust_string(&jvm, "audio/second").await?;
            let second_clip: ClassInstanceRef<Clip> = jvm
                .new_class("org/kwis/msp/media/Clip", "(Ljava/lang/String;)V", (second_type,))
                .await?
                .into();

            let second_initial_volume: i32 = jvm.invoke_virtual(&second_clip, "getVolume", "()I", ()).await?;

            let minimum_set: bool = jvm.invoke_virtual(&clip, "setVolume", "(I)Z", (0,)).await?;
            let minimum_volume: i32 = jvm.invoke_virtual(&clip, "getVolume", "()I", ()).await?;
            let maximum_set: bool = jvm.invoke_virtual(&clip, "setVolume", "(I)Z", (100,)).await?;
            let maximum_volume: i32 = jvm.invoke_virtual(&clip, "getVolume", "()I", ()).await?;
            let below_minimum_set: bool = jvm.invoke_virtual(&clip, "setVolume", "(I)Z", (-1,)).await?;
            let volume_after_below_minimum: i32 = jvm.invoke_virtual(&clip, "getVolume", "()I", ()).await?;
            let above_maximum_set: bool = jvm.invoke_virtual(&clip, "setVolume", "(I)Z", (101,)).await?;
            let volume_after_above_maximum: i32 = jvm.invoke_virtual(&clip, "getVolume", "()I", ()).await?;
            let second_final_volume: i32 = jvm.invoke_virtual(&second_clip, "getVolume", "()I", ()).await?;

            assert_eq!(second_initial_volume, 0);
            assert!(minimum_set);
            assert_eq!(minimum_volume, 0);
            assert!(maximum_set);
            assert_eq!(maximum_volume, 100);
            assert!(!below_minimum_set);
            assert_eq!(volume_after_below_minimum, 100);
            assert!(!above_maximum_set);
            assert_eq!(volume_after_above_maximum, 100);
            assert_eq!(second_final_volume, 0);

            Ok(())
        })
    }
}
