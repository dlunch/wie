use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::{
    io::{DataInputStream, InputStream},
    lang::String,
};
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[repr(i32)]
#[derive(Eq, PartialEq)]
#[allow(dead_code)]
enum Mode {
    // wipi constant
    READ_ONLY = 1,
    WRITE = 2,
    WRITE_TRUNC = 3,
    READ_WRITE = 4,
}

// class org.kwis.msp.io.File
pub struct File;

impl File {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/io/File",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;II)V", Self::init_with_flag, Default::default()),
                JavaMethodProto::new("write", "([B)I", Self::write, Default::default()),
                JavaMethodProto::new("write", "([BII)I", Self::write_with_offset_length, Default::default()),
                JavaMethodProto::new("read", "([B)I", Self::read, Default::default()),
                JavaMethodProto::new("read", "([BII)I", Self::read_with_offset_length, Default::default()),
                JavaMethodProto::new("seek", "(I)V", Self::seek, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
                JavaMethodProto::new("sizeOf", "()I", Self::size_of, Default::default()),
                JavaMethodProto::new("openInputStream", "()Ljava/io/InputStream;", Self::open_input_stream, Default::default()),
                JavaMethodProto::new(
                    "openDataInputStream",
                    "()Ljava/io/DataInputStream;",
                    Self::open_data_input_stream,
                    Default::default(),
                ),
            ],
            fields: vec![
                JavaFieldProto::new("file", "Ljava/io/File;", Default::default()),
                JavaFieldProto::new("raf", "Ljava/io/RandomAccessFile;", Default::default()),
            ],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, filename: ClassInstanceRef<String>, mode: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({:?}, {:?}, {:?})", &this, &filename, mode);

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/io/File", "<init>", "(Ljava/lang/String;II)V", (filename, mode, 0))
            .await?;

        Ok(())
    }

    async fn init_with_flag(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        filename: ClassInstanceRef<String>,
        mode: i32,
        flag: i32,
    ) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({:?}, {:?}, {:?}, {:?})", &this, &filename, mode, flag);

        let name = JavaLangString::to_rust_string(jvm, &filename).await?;
        if name.is_empty() {
            return Err(jvm.exception("java/io/IOException", "Invalid filename").await);
        }

        let mode = unsafe { core::mem::transmute::<i32, Mode>(mode) };

        let mode_string = if mode == Mode::WRITE || mode == Mode::WRITE_TRUNC { "w" } else { "rw" };
        let mode_string = JavaLangString::from_rust_string(jvm, mode_string).await?;

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (filename,)).await?;

        let raf = jvm
            .new_class(
                "java/io/RandomAccessFile",
                "(Ljava/io/File;Ljava/lang/String;)V",
                (file.clone(), mode_string),
            )
            .await?;

        if mode == Mode::WRITE_TRUNC {
            let _: () = jvm.invoke_virtual(&raf, "setLength", "(J)V", (0i64,)).await?;
        }

        jvm.put_field(&mut this, "raf", "Ljava/io/RandomAccessFile;", raf).await?;
        jvm.put_field(&mut this, "file", "Ljava/io/File;", file).await?;

        Ok(())
    }

    async fn write(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, buf: ClassInstanceRef<Array<i8>>) -> JvmResult<i32> {
        let length = jvm.array_length(&buf).await? as i32;

        jvm.invoke_virtual(&this, "write", "([BII)I", (buf, 0, length)).await
    }

    async fn write_with_offset_length(
        _jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<ClassInstanceRef<Array<i8>>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.File::write({:?}, {:?}, {:?}, {:?})", &this, &buf, offset, len);

        Ok(0)
    }

    async fn seek(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, pos: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::seek({:?}, {:?})", &this, pos);

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "seek", "(J)V", (pos as i64,)).await?;

        Ok(())
    }

    async fn read(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, buf: ClassInstanceRef<Array<i8>>) -> JvmResult<i32> {
        let length = jvm.array_length(&buf).await? as i32;

        jvm.invoke_virtual(&this, "read", "([BII)I", (buf, 0, length)).await
    }

    async fn read_with_offset_length(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::read({:?}, {:?})", &this, &buf);

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let read = jvm.invoke_virtual(&raf, "read", "([BII)I", (buf, offset, length)).await?;

        Ok(read)
    }

    async fn close(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::close({:?})", &this);

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "close", "()V", ()).await?;

        Ok(())
    }

    async fn size_of(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::sizeOf({:?})", &this);

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;

        Ok(length as _)
    }

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<InputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openInputStream({:?})", &this);

        let file: ClassInstanceRef<File> = jvm.get_field(&this, "file", "Ljava/io/File;").await?;
        let input_stream = jvm.new_class("java/io/FileInputStream", "(Ljava/io/File;)V", (file,)).await?;

        Ok(input_stream.into())
    }

    async fn open_data_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<DataInputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openDataInputStream({:?})", &this);

        let input_stream: ClassInstanceRef<InputStream> = jvm.invoke_virtual(&this, "openInputStream", "()Ljava/io/InputStream;", ()).await?;
        let data_input_stream = jvm
            .new_class("java/io/DataInputStream", "(Ljava/io/InputStream;)V", (input_stream,))
            .await?;

        Ok(data_input_stream.into())
    }
}
