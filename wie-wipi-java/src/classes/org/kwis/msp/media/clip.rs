use bytemuck::cast_vec;

use jvm::{
    Array, ClassInstanceRef, Jvm, Result as JvmResult,
    runtime::{JavaIoInputStream, JavaLangString},
};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::{JvmSupport, WieJvmContext};
use wie_midp::classes::javax::microedition::media::Player;

use crate::classes::org::kwis::msp::media::PlayListener;

// class org.kwis.msp.media.Clip
#[java_class(name = "org/kwis/msp/media/Clip", extends = "org/kwis/msp/media/BaseClip", public)]
pub struct Clip {
    #[field(private)]
    position: i32,
    #[field(private)]
    stopTime: i32,
    #[field(private)]
    r#type: ClassInstanceRef<String>,
    #[field(private)]
    volume: i32,
}

#[java_methods]
impl Clip {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, r#type: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?})");

        let _: () = jvm.invoke_special(&this, "org/kwis/msp/media/BaseClip", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "org/kwis/msp/media/Clip", "type", "Ljava/lang/String;", r#type)
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_resource(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: ClassInstanceRef<String>,
        resource_name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::<init>({this:?}, {type:?}, {resource_name:?})");

        let name = JavaLangString::to_rust_string(jvm, &resource_name).await?;
        let resource_stream: ClassInstanceRef<InputStream> = JvmSupport::open_resource(jvm, &name).await?.into();
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

    #[method(constructor, public)]
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

        let _: () = jvm
            .invoke_virtual(&this, "org/kwis/msp/media/Clip", "setBuffer", "([BI)V", (data, length as i32))
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
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

    #[method(public)]
    async fn set_volume(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Clip>, level: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setVolume({this:?}, {level})");

        if !(0..=100).contains(&level) {
            return Ok(false);
        }

        jvm.put_field(&mut this, "org/kwis/msp/media/Clip", "volume", "I", level).await?;

        Ok(true)
    }

    // not in spec, but some apps call it
    #[method(name = "setVolume", public)]
    async fn set_volume_void(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Clip>, level: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::setVolume({this:?}, {level})");

        let _: bool = jvm
            .invoke_virtual(&this, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (level,))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn set_listener(_: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, listener: ClassInstanceRef<PlayListener>) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.media.Clip::setListener({this:?}, {listener:?})");

        Ok(())
    }

    #[method(public)]
    async fn set_buffer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.Clip::setBuffer({this:?}, {buffer:?}, {size})");

        let _: bool = jvm
            .invoke_virtual(&this, "org/kwis/msp/media/BaseClip", "setBuffer", "([BI)Z", (buffer, size))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn get_type(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::debug!("org.kwis.msp.media.Clip::getType({this:?})");

        jvm.get_field(&this, "org/kwis/msp/media/Clip", "type", "Ljava/lang/String;").await
    }

    #[method(public)]
    async fn set_position(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, milliseconds: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setPosition({this:?}, {milliseconds})");

        jvm.put_field(&mut this, "org/kwis/msp/media/Clip", "position", "I", milliseconds).await?;

        Ok(true)
    }

    #[method(public)]
    async fn get_position(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getPosition({this:?})");

        jvm.get_field(&this, "org/kwis/msp/media/Clip", "position", "I").await
    }

    #[method(public)]
    async fn set_stop_time(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, milliseconds: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.Clip::setStopTime({this:?}, {milliseconds})");

        jvm.put_field(&mut this, "org/kwis/msp/media/Clip", "stopTime", "I", milliseconds).await?;

        Ok(true)
    }

    #[method(public)]
    async fn get_stop_time(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getStopTime({this:?})");

        jvm.get_field(&this, "org/kwis/msp/media/Clip", "stopTime", "I").await
    }

    #[method(public)]
    async fn get_volume(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::getVolume({this:?})");

        jvm.get_field(&this, "org/kwis/msp/media/Clip", "volume", "I").await
    }

    #[method]
    async fn get_or_create_player(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Player>> {
        let player: ClassInstanceRef<Player> = jvm
            .get_field(&this, "org/kwis/msp/media/BaseClip", "player", "Ljavax/microedition/media/Player;")
            .await?;
        if !player.is_null() {
            return Ok(player);
        }

        let stream: ClassInstanceRef<InputStream> = jvm
            .get_field(&this, "org/kwis/msp/media/BaseClip", "stream", "Ljava/io/InputStream;")
            .await?;
        if stream.is_null() {
            return Ok(None.into());
        }

        let r#type = JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?;
        let player: ClassInstanceRef<Player> = jvm
            .invoke_static(
                "javax/microedition/media/Manager",
                "createPlayer",
                "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                (stream, r#type),
            )
            .await?;
        jvm.put_field(
            &mut this,
            "org/kwis/msp/media/BaseClip",
            "player",
            "Ljavax/microedition/media/Player;",
            player.clone(),
        )
        .await?;

        Ok(player)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{ClassInstanceRef, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::org::kwis::msp::media::Clip, get_protos};

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

            let second_initial_volume: i32 = jvm
                .invoke_virtual(&second_clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ())
                .await?;

            let minimum_set: bool = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (0,)).await?;
            let minimum_volume: i32 = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ()).await?;
            let maximum_set: bool = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (100,)).await?;
            let maximum_volume: i32 = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ()).await?;
            let below_minimum_set: bool = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (-1,)).await?;
            let volume_after_below_minimum: i32 = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ()).await?;
            let above_maximum_set: bool = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "setVolume", "(I)Z", (101,)).await?;
            let volume_after_above_maximum: i32 = jvm.invoke_virtual(&clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ()).await?;
            let second_final_volume: i32 = jvm
                .invoke_virtual(&second_clip, "org/kwis/msp/media/Clip", "getVolume", "()I", ())
                .await?;

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
