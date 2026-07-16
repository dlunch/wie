use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::io::RandomAccessFile;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::io::File;

// Internal stream used to preserve the WIPI File's position and stream lifecycle.
pub struct WIPIFileOutputStream;

impl WIPIFileOutputStream {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WIPIFileOutputStream",
            parent_class: Some("java/io/OutputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/io/File;)V", Self::init, Default::default()),
                JavaMethodProto::new("write", "(I)V", Self::write, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("file", "Lorg/kwis/msp/io/File;", Default::default()),
                JavaFieldProto::new("closed", "Z", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, file: ClassInstanceRef<File>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::<init>({this:?}, {file:?})");

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "file", "Lorg/kwis/msp/io/File;", file).await?;
        jvm.put_field(&mut this, "closed", "Z", false).await?;

        Ok(())
    }

    async fn write(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, byte: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::write({this:?}, {byte})");

        let stream_closed: bool = jvm.get_field(&this, "closed", "Z").await?;
        let file: ClassInstanceRef<File> = jvm.get_field(&this, "file", "Lorg/kwis/msp/io/File;").await?;
        let file_closed: bool = jvm.get_field(&file, "closed", "Z").await?;
        if stream_closed || file_closed {
            return Err(jvm.exception("java/io/IOException", "Stream closed").await);
        }

        let mut buffer = jvm.instantiate_array("B", 1).await?;
        jvm.store_array(&mut buffer, 0, [byte as i8]).await?;

        let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&file, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (buffer, 0, 1)).await?;

        Ok(())
    }

    async fn close(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::close({this:?})");

        let closed: bool = jvm.get_field(&this, "closed", "Z").await?;
        if closed {
            return Ok(());
        }

        let mut file: ClassInstanceRef<File> = jvm.get_field(&this, "file", "Lorg/kwis/msp/io/File;").await?;
        jvm.put_field(&mut file, "outputStreamOpen", "Z", false).await?;
        jvm.put_field(&mut this, "closed", "Z", true).await?;

        Ok(())
    }
}
