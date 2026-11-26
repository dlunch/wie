use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::io::InputStream;

use wie_jvm_support::WieJvmContext;
use wie_midp::classes::javax::microedition::media::Player;

// not in reference, but called by some apps..
// class org.kwis.msp.media.BaseClip
#[java_class(name = "org/kwis/msp/media/BaseClip", public)]
pub struct BaseClip {
    #[field(private)]
    stream: ClassInstanceRef<InputStream>,
    #[field(private)]
    player: ClassInstanceRef<Player>,
}

#[java_methods]
impl BaseClip {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.BaseClip::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(public)]
    async fn available_data_size(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.BaseClip::availableDataSize({this:?})");

        Ok(10000000 as _)
    }

    #[method(public)]
    async fn put_data(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::putData({this:?}, {buffer:?}, {offset}, {length})");

        let data = jvm.load_array::<i8>(&buffer, offset as usize, length as usize).await?;
        let mut copied = jvm.instantiate_array("B", data.len()).await?;
        jvm.store_array(&mut copied, 0, data).await?;
        let _: bool = jvm
            .invoke_virtual(&this, "org/kwis/msp/media/BaseClip", "setBuffer", "([BI)Z", (copied, length))
            .await?;

        Ok(length)
    }

    #[method(public)]
    async fn clear_data(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.BaseClip::clearData({this:?})");

        jvm.put_field(&mut this, "org/kwis/msp/media/BaseClip", "stream", "Ljava/io/InputStream;", None)
            .await?;

        let player: ClassInstanceRef<Player> = jvm
            .get_field(&this, "org/kwis/msp/media/BaseClip", "player", "Ljavax/microedition/media/Player;")
            .await?;
        if player.is_null() {
            return Ok(());
        }

        let _: () = jvm.invoke_virtual(&player, "javax/microedition/media/Player", "close", "()V", ()).await?;

        jvm.put_field(
            &mut this,
            "org/kwis/msp/media/BaseClip",
            "player",
            "Ljavax/microedition/media/Player;",
            None,
        )
        .await?;

        Ok(())
    }

    #[method(public)]
    async fn set_buffer(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.BaseClip::setBuffer({this:?}, {buffer:?}, {size})");

        let stream = jvm.new_class("java/io/ByteArrayInputStream", "([BII)V", (buffer, 0, size)).await?;
        let _: () = jvm.invoke_virtual(&this, "org/kwis/msp/media/BaseClip", "clearData", "()V", ()).await?;
        jvm.put_field(&mut this, "org/kwis/msp/media/BaseClip", "stream", "Ljava/io/InputStream;", stream)
            .await?;

        Ok(true)
    }
}
