use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::io::RandomAccessFile;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::org::kwis::msp::io::File;

// Internal stream used to preserve the WIPI File's position.
pub struct WIPIFileInputStream;

impl WIPIFileInputStream {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "net/wie/WIPIFileInputStream",
            parent_class: Some("java/io/InputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Lorg/kwis/msp/io/File;)V", Self::init, Default::default()),
                JavaMethodProto::new("read", "()I", Self::read, Default::default()),
                JavaMethodProto::new("read", "([BII)I", Self::read_with_offset_length, Default::default()),
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
        tracing::debug!("net.wie.WIPIFileInputStream::<init>({this:?}, {file:?})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        jvm.put_field(&mut this, "file", "Lorg/kwis/msp/io/File;", file).await?;
        jvm.put_field(&mut this, "closed", "Z", false).await?;

        Ok(())
    }

    async fn file(jvm: &Jvm, this: &ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<File>> {
        let stream_closed: bool = jvm.get_field(this, "closed", "Z").await?;
        let file: ClassInstanceRef<File> = jvm.get_field(this, "file", "Lorg/kwis/msp/io/File;").await?;
        let file_closed: bool = jvm.get_field(&file, "closed", "Z").await?;
        if stream_closed || file_closed {
            return Err(jvm.exception("java/io/IOException", "Stream closed").await);
        }

        Ok(file)
    }

    async fn read(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("net.wie.WIPIFileInputStream::read({this:?})");

        let file = Self::file(jvm, &this).await?;
        jvm.invoke_virtual(&file, "read", "()I", ()).await
    }

    async fn read_with_offset_length(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("net.wie.WIPIFileInputStream::read({this:?}, {buffer:?}, {offset}, {length})");

        let array_length = jvm.array_length(&buffer).await? as i32;
        if offset < 0 || length < 0 || offset > array_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(0);
        }

        let file = Self::file(jvm, &this).await?;
        let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&file, "raf", "Ljava/io/RandomAccessFile;").await?;
        jvm.invoke_virtual(&raf, "read", "([BII)I", (buffer, offset, length)).await
    }

    async fn close(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WIPIFileInputStream::close({this:?})");

        jvm.put_field(&mut this, "closed", "Z", true).await
    }
}
