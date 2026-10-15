use alloc::vec;

use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{JavaFieldProto, JavaMethodProto};
use jvm_types::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::media::Player;

// not in reference, but called by some apps..
// class org.kwis.msp.media.BaseClip
pub struct BaseClip;

impl BaseClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/BaseClip",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("putData", "([BII)I", Self::put_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("setBuffer", "([BI)Z", Self::set_buffer, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("clearData", "()V", Self::clear_data, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("availableDataSize", "()I", Self::available_data_size, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![
                JavaFieldProto::new("stream", "Ljava/io/InputStream;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("player", "Ljavax/microedition/media/Player;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.media.BaseClip::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn available_data_size(_jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.media.BaseClip::availableDataSize({this:?})");

        Ok(10000000 as _)
    }

    async fn put_data(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.media.Clip::putData({this:?}, {buffer:?}, {offset}, {length})");

        let data = jvm.load_array::<i8>(&buffer, offset as usize, length as usize).await?;
        let mut copied = jvm.instantiate_array("B", data.len()).await?;
        jvm.store_array(&mut copied, 0, data).await?;
        Self::set_buffer(jvm, context, this, copied.into(), length).await?;

        Ok(length)
    }

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

    async fn set_buffer(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        size: i32,
    ) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.media.BaseClip::setBuffer({this:?}, {buffer:?}, {size})");

        let stream = jvm.new_class("java/io/ByteArrayInputStream", "([BII)V", (buffer, 0, size)).await?;
        Self::clear_data(jvm, context, this.clone()).await?;
        jvm.put_field(&mut this, "org/kwis/msp/media/BaseClip", "stream", "Ljava/io/InputStream;", stream)
            .await?;

        Ok(true)
    }
}
