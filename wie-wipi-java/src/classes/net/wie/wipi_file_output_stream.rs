use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::io::RandomAccessFile;

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msp::io::File;

// Internal stream used to preserve the WIPI File's position and stream lifecycle.
#[java_class(name = "net/wie/WIPIFileOutputStream", extends = "java/io/OutputStream", public)]
pub struct WIPIFileOutputStream {
    #[field(private)]
    file: ClassInstanceRef<File>,
    #[field(private)]
    closed: bool,
}

#[java_methods]
impl WIPIFileOutputStream {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, file: ClassInstanceRef<File>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::<init>({this:?}, {file:?})");

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "net/wie/WIPIFileOutputStream", "file", "Lorg/kwis/msp/io/File;", file)
            .await?;
        jvm.put_field(&mut this, "net/wie/WIPIFileOutputStream", "closed", "Z", false).await?;

        Ok(())
    }

    #[method(public)]
    async fn write(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, byte: i32) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::write({this:?}, {byte})");

        let stream_closed: bool = jvm.get_field(&this, "net/wie/WIPIFileOutputStream", "closed", "Z").await?;
        let file: ClassInstanceRef<File> = jvm
            .get_field(&this, "net/wie/WIPIFileOutputStream", "file", "Lorg/kwis/msp/io/File;")
            .await?;
        let file_closed: bool = jvm.get_field(&file, "org/kwis/msp/io/File", "closed", "Z").await?;
        if stream_closed || file_closed {
            return Err(jvm.exception("java/io/IOException", "Stream closed").await);
        }

        let mut buffer = jvm.instantiate_array("B", 1).await?;
        jvm.store_array(&mut buffer, 0, [byte as i8]).await?;

        let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&file, "org/kwis/msp/io/File", "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm
            .invoke_virtual(&raf, "java/io/RandomAccessFile", "write", "([BII)V", (buffer, 0, 1))
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn close(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileOutputStream::close({this:?})");

        let closed: bool = jvm.get_field(&this, "net/wie/WIPIFileOutputStream", "closed", "Z").await?;
        if closed {
            return Ok(());
        }

        let mut file: ClassInstanceRef<File> = jvm
            .get_field(&this, "net/wie/WIPIFileOutputStream", "file", "Lorg/kwis/msp/io/File;")
            .await?;
        jvm.put_field(&mut file, "org/kwis/msp/io/File", "outputStreamOpen", "Z", false).await?;
        jvm.put_field(&mut this, "net/wie/WIPIFileOutputStream", "closed", "Z", true).await?;

        Ok(())
    }
}
