use alloc::vec;

use jvm::{
    Array, ClassInstanceRef, Jvm, Result,
    runtime::{JavaIoInputStream, JavaLangString},
};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::media::{Control, PlayerListener};

// class net.wie.SmafPlayer
pub struct SmafPlayer;

impl SmafPlayer {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/SmafPlayer",
            parent_class: Some("java/lang/Object"),
            interfaces: vec!["javax/microedition/media/Player"],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/io/InputStream;)V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("realize", "()V", Self::realize, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("prefetch", "()V", Self::prefetch, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("start", "()V", Self::start, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("start", "(Z)V", Self::start_with_repeat, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("stop", "()V", Self::stop, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("deallocate", "()V", Self::deallocate, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("close", "()V", Self::close, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setMediaTime", "(J)J", Self::set_media_time, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getMediaTime", "()J", Self::get_media_time, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getState", "()I", Self::get_state, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("getDuration", "()J", Self::get_duration, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "getContentType",
                    "()Ljava/lang/String;",
                    Self::get_content_type,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new("setLoopCount", "(I)V", Self::set_loop_count, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new(
                    "addPlayerListener",
                    "(Ljavax/microedition/media/PlayerListener;)V",
                    Self::add_player_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "removePlayerListener",
                    "(Ljavax/microedition/media/PlayerListener;)V",
                    Self::remove_player_listener,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    Self::get_control,
                    MethodAccessFlags::PUBLIC,
                ),
                JavaMethodProto::new(
                    "getControls",
                    "()[Ljavax/microedition/media/Control;",
                    Self::get_controls,
                    MethodAccessFlags::PUBLIC,
                ),
            ],
            fields: vec![JavaFieldProto::new("audioHandle", "I", FieldAccessFlags::PRIVATE)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::<init>({this:?}, {stream:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let audio_handle = context.system().audio().load_smaf(&data).unwrap();

        jvm.put_field(&mut this, "audioHandle", "I", audio_handle as i32).await?;

        Ok(())
    }

    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        Self::start_with_repeat(jvm, context, this, false).await
    }

    async fn start_with_repeat(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, repeat: bool) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::start({this:?}, {repeat})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        context.system().audio().play(audio_handle as u32, repeat).unwrap();

        Ok(())
    }

    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::stop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        let system = context.system();

        system.audio().stop(audio_handle as u32);

        Ok(())
    }

    async fn close(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::close({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "audioHandle", "I").await?;

        let system = context.system();

        system.audio().close(audio_handle as u32).unwrap();

        Ok(())
    }

    async fn realize(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::realize({this:?})");

        Ok(())
    }

    async fn prefetch(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::prefetch({this:?})");

        Ok(())
    }

    async fn deallocate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::deallocate({this:?})");

        Ok(())
    }

    async fn set_media_time(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, now: i64) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::setMediaTime({this:?}, {now})");

        Err(jvm.exception("javax/microedition/media/MediaException", "Seeking is not supported").await)
    }

    async fn get_media_time(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getMediaTime({this:?})");

        Ok(-1)
    }

    async fn get_state(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32> {
        tracing::warn!("stub net.wie.SmafPlayer::getState({this:?})");

        Ok(300)
    }

    async fn get_duration(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getDuration({this:?})");

        Ok(-1)
    }

    async fn get_content_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<String>> {
        tracing::debug!("net.wie.SmafPlayer::getContentType({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?.into())
    }

    async fn set_loop_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, count: i32) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::setLoopCount({this:?}, {count})");

        if count == 0 || count < -1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid loop count").await);
        }

        Ok(())
    }

    async fn add_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::addPlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn remove_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::removePlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    async fn get_control(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        control_type: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Control>> {
        tracing::warn!("stub net.wie.SmafPlayer::getControl({this:?}, {control_type:?})");

        if control_type.is_null() {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Control type is null").await);
        }

        Ok(None.into())
    }

    async fn get_controls(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<Control>>> {
        tracing::warn!("stub net.wie.SmafPlayer::getControls({this:?})");

        Ok(jvm.instantiate_array("Ljavax/microedition/media/Control;", 0).await?.into())
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, JavaError, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{
        classes::javax::microedition::media::{Control, Player},
        get_protos,
    };

    #[test]
    fn test_unsupported_player_controls() -> Result<()> {
        run_jvm_test(Box::new([get_protos().into()]), |jvm| async move {
            let data = jvm.instantiate_array("B", 0).await?;
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (data,)).await?;
            let content_type = JavaLangString::from_rust_string(&jvm, "application/vnd.smaf").await?;
            let player: ClassInstanceRef<Player> = jvm
                .invoke_static(
                    "javax/microedition/media/Manager",
                    "createPlayer",
                    "(Ljava/io/InputStream;Ljava/lang/String;)Ljavax/microedition/media/Player;",
                    (stream, content_type),
                )
                .await?;
            let _: () = jvm
                .invoke_virtual(&player, "javax/microedition/media/Player", "realize", "()V", ())
                .await?;

            let controls: ClassInstanceRef<Array<Control>> = jvm
                .invoke_virtual(
                    &player,
                    "javax/microedition/media/Controllable",
                    "getControls",
                    "()[Ljavax/microedition/media/Control;",
                    (),
                )
                .await?;
            assert_eq!(jvm.array_length(&controls).await?, 0);

            let control_type = JavaLangString::from_rust_string(&jvm, "VolumeControl").await?;
            let control: ClassInstanceRef<Control> = jvm
                .invoke_virtual(
                    &player,
                    "javax/microedition/media/Player",
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    (control_type,),
                )
                .await?;
            assert!(control.is_null());

            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, ClassInstanceRef<Control>>(
                    &player,
                    "javax/microedition/media/Controllable",
                    "getControl",
                    "(Ljava/lang/String;)Ljavax/microedition/media/Control;",
                    (None,),
                )
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let _: () = jvm
                .invoke_virtual(&player, "javax/microedition/media/Player", "setLoopCount", "(I)V", (-1,))
                .await?;
            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, ()>(&player, "javax/microedition/media/Player", "setLoopCount", "(I)V", (0,))
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "java/lang/IllegalArgumentException"));

            let JavaError::JavaException(exception) = jvm
                .invoke_virtual::<_, i64>(&player, "javax/microedition/media/Player", "setMediaTime", "(J)J", (0i64,))
                .await
                .unwrap_err();
            assert!(jvm.is_instance(&*exception, "javax/microedition/media/MediaException"));

            let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;
            Ok(())
        })
    }
}
