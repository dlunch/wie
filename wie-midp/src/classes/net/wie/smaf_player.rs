use jvm::{
    Array, ClassInstanceRef, Jvm, Result,
    runtime::{JavaIoInputStream, JavaLangString},
};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::media::{Control, PlayerListener};

// class net.wie.SmafPlayer
#[java_class(name = "net/wie/SmafPlayer", implements("javax/microedition/media/Player"), public)]
pub struct SmafPlayer {
    #[field(private)]
    audioHandle: i32,
}

#[java_methods]
impl SmafPlayer {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, stream: ClassInstanceRef<InputStream>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::<init>({this:?}, {stream:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        let data = JavaIoInputStream::read_until_end(jvm, &stream).await?;
        let audio_handle = context.system().audio().load_smaf(&data).unwrap();

        jvm.put_field(&mut this, "net/wie/SmafPlayer", "audioHandle", "I", audio_handle as i32)
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn start(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        Self::start_with_repeat(jvm, context, this, false).await
    }

    #[method(name = "start", public)]
    async fn start_with_repeat(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>, repeat: bool) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::start({this:?}, {repeat})");

        let audio_handle: i32 = jvm.get_field(&this, "net/wie/SmafPlayer", "audioHandle", "I").await?;

        context.system().audio().play(audio_handle as u32, repeat).unwrap();

        Ok(())
    }

    #[method(public)]
    async fn stop(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::stop({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "net/wie/SmafPlayer", "audioHandle", "I").await?;

        let system = context.system();

        system.audio().stop(audio_handle as u32);

        Ok(())
    }

    #[method(public)]
    async fn close(jvm: &Jvm, context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.SmafPlayer::close({this:?})");

        let audio_handle: i32 = jvm.get_field(&this, "net/wie/SmafPlayer", "audioHandle", "I").await?;

        let system = context.system();

        system.audio().close(audio_handle as u32).unwrap();

        Ok(())
    }

    #[method(public)]
    async fn realize(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::realize({this:?})");

        Ok(())
    }

    #[method(public)]
    async fn prefetch(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::prefetch({this:?})");

        Ok(())
    }

    #[method(public)]
    async fn deallocate(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::deallocate({this:?})");

        Ok(())
    }

    #[method(public)]
    async fn set_media_time(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, now: i64) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::setMediaTime({this:?}, {now})");

        Err(jvm.exception("javax/microedition/media/MediaException", "Seeking is not supported").await)
    }

    #[method(public)]
    async fn get_media_time(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getMediaTime({this:?})");

        Ok(-1)
    }

    #[method(public)]
    async fn get_state(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32> {
        tracing::warn!("stub net.wie.SmafPlayer::getState({this:?})");

        Ok(300)
    }

    #[method(public)]
    async fn get_duration(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i64> {
        tracing::warn!("stub net.wie.SmafPlayer::getDuration({this:?})");

        Ok(-1)
    }

    #[method(public)]
    async fn get_content_type(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<String>> {
        tracing::debug!("net.wie.SmafPlayer::getContentType({this:?})");

        Ok(JavaLangString::from_rust_string(jvm, "application/vnd.smaf").await?.into())
    }

    #[method(public)]
    async fn set_loop_count(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, count: i32) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::setLoopCount({this:?}, {count})");

        if count == 0 || count < -1 {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid loop count").await);
        }

        Ok(())
    }

    #[method(public)]
    async fn add_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::addPlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    #[method(public)]
    async fn remove_player_listener(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        listener: ClassInstanceRef<PlayerListener>,
    ) -> Result<()> {
        tracing::warn!("stub net.wie.SmafPlayer::removePlayerListener({this:?}, {listener:?})");

        Ok(())
    }

    #[method(public)]
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

    #[method(public)]
    async fn get_controls(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<Control>>> {
        tracing::warn!("stub net.wie.SmafPlayer::getControls({this:?})");

        Ok(jvm.instantiate_array("Ljavax/microedition/media/Control;", 0).await?.into())
    }
}
