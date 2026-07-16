use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::{
    io::{DataInputStream, DataOutputStream, File as JavaFile, InputStream, OutputStream},
    lang::String,
};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

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

impl Mode {
    fn from_raw(raw: i32) -> Option<Self> {
        Some(match raw {
            x if x == Self::READ_ONLY as i32 => Self::READ_ONLY,
            x if x == Self::WRITE as i32 => Self::WRITE,
            x if x == Self::WRITE_TRUNC as i32 => Self::WRITE_TRUNC,
            x if x == Self::READ_WRITE as i32 => Self::READ_WRITE,
            _ => return None,
        })
    }
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
                JavaMethodProto::new(
                    "openOutputStream",
                    "()Ljava/io/OutputStream;",
                    Self::open_output_stream,
                    Default::default(),
                ),
                JavaMethodProto::new(
                    "openDataOutputStream",
                    "()Ljava/io/DataOutputStream;",
                    Self::open_data_output_stream,
                    Default::default(),
                ),
                JavaMethodProto::new("write", "(I)I", Self::write_byte, Default::default()),
                JavaMethodProto::new("read", "()I", Self::read_byte, Default::default()),
                JavaMethodProto::new("tell", "()I", Self::tell, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("file", "Ljava/io/File;", Default::default()),
                JavaFieldProto::new("raf", "Ljava/io/RandomAccessFile;", Default::default()),
                JavaFieldProto::new("mode", "I", Default::default()),
                JavaFieldProto::new("closed", "Z", Default::default()),
                JavaFieldProto::new("outputStreamOpen", "Z", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, filename: ClassInstanceRef<String>, mode: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::<init>({this:?}, {filename:?}, {mode:?})");

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
        tracing::debug!("org.kwis.msp.io.File::<init>({this:?}, {filename:?}, {mode:?}, {flag:?})");

        let name = JavaLangString::to_rust_string(jvm, &filename).await?;
        if name.is_empty() {
            return Err(jvm.exception("java/io/IOException", "Invalid filename").await);
        }

        let mode = if let Some(mode) = Mode::from_raw(mode) {
            mode
        } else {
            return Err(jvm.exception("java/io/IOException", "Invalid mode").await);
        };

        let mode_string = if mode == Mode::READ_ONLY { "r" } else { "w" };
        let mode_string = JavaLangString::from_rust_string(jvm, mode_string).await?;

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (filename,)).await?;

        let raf = jvm
            .new_class(
                "java/io/RandomAccessFile",
                "(Ljava/io/File;Ljava/lang/String;)V",
                (file.clone(), mode_string),
            )
            .await;

        if raf.is_err() {
            // TODO check exception type
            return Err(jvm.exception("java/io/IOException", "Invalid filename").await);
        }
        let raf = raf.unwrap();

        if mode == Mode::WRITE_TRUNC {
            let _: () = jvm.invoke_virtual(&raf, "setLength", "(J)V", (0i64,)).await?;
        }

        jvm.put_field(&mut this, "raf", "Ljava/io/RandomAccessFile;", raf).await?;
        jvm.put_field(&mut this, "file", "Ljava/io/File;", file).await?;
        jvm.put_field(&mut this, "mode", "I", mode as i32).await?;
        jvm.put_field(&mut this, "closed", "Z", false).await?;
        jvm.put_field(&mut this, "outputStreamOpen", "Z", false).await?;

        Ok(())
    }

    async fn write(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, buf: ClassInstanceRef<Array<i8>>) -> JvmResult<i32> {
        let length = jvm.array_length(&buf).await? as i32;

        jvm.invoke_virtual(&this, "write", "([BII)I", (buf, 0, length)).await
    }

    async fn write_with_offset_length(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<ClassInstanceRef<Array<i8>>>,
        offset: i32,
        len: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::write({this:?}, {buf:?}, {offset:?}, {len:?})");

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (buf, offset, len)).await?;

        Ok(0)
    }

    async fn seek(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, pos: i32) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::seek({this:?}, {pos:?})");

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
        tracing::debug!("org.kwis.msp.io.File::read({this:?}, {buf:?})");

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let read = jvm.invoke_virtual(&raf, "read", "([BII)I", (buf, offset, length)).await?;

        Ok(read)
    }

    async fn close(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("org.kwis.msp.io.File::close({this:?})");

        let closed: bool = jvm.get_field(&this, "closed", "Z").await?;
        if closed {
            return Ok(());
        }

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "close", "()V", ()).await?;
        jvm.put_field(&mut this, "closed", "Z", true).await?;

        Ok(())
    }

    async fn size_of(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::sizeOf({this:?})");

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;

        Ok(length as _)
    }

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<InputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openInputStream({this:?})");

        let file: ClassInstanceRef<JavaFile> = jvm.get_field(&this, "file", "Ljava/io/File;").await?;
        let input_stream = jvm.new_class("java/io/FileInputStream", "(Ljava/io/File;)V", (file,)).await?;

        Ok(input_stream.into())
    }

    async fn open_data_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<DataInputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openDataInputStream({this:?})");

        let input_stream: ClassInstanceRef<InputStream> = jvm.invoke_virtual(&this, "openInputStream", "()Ljava/io/InputStream;", ()).await?;
        let data_input_stream = jvm
            .new_class("java/io/DataInputStream", "(Ljava/io/InputStream;)V", (input_stream,))
            .await?;

        Ok(data_input_stream.into())
    }

    async fn open_output_stream(jvm: &Jvm, _: &mut WieJvmContext, mut this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<OutputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openOutputStream({this:?})");

        let closed: bool = jvm.get_field(&this, "closed", "Z").await?;
        if closed {
            return Err(jvm.exception("java/io/IOException", "File closed").await);
        }

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode == Mode::READ_ONLY as i32 {
            return Err(jvm.exception("java/io/IOException", "File is read-only").await);
        }

        let output_stream_open: bool = jvm.get_field(&this, "outputStreamOpen", "Z").await?;
        if output_stream_open {
            return Err(jvm.exception("java/io/IOException", "Output stream already open").await);
        }

        let output_stream = jvm
            .new_class("net/wie/WIPIFileOutputStream", "(Lorg/kwis/msp/io/File;)V", (this.clone(),))
            .await?;
        jvm.put_field(&mut this, "outputStreamOpen", "Z", true).await?;

        Ok(output_stream.into())
    }

    async fn open_data_output_stream(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
    ) -> JvmResult<ClassInstanceRef<DataOutputStream>> {
        tracing::debug!("org.kwis.msp.io.File::openDataOutputStream({this:?})");

        let output_stream: ClassInstanceRef<OutputStream> = jvm.invoke_virtual(&this, "openOutputStream", "()Ljava/io/OutputStream;", ()).await?;
        let data_output_stream = jvm
            .new_class("java/io/DataOutputStream", "(Ljava/io/OutputStream;)V", (output_stream,))
            .await?;

        Ok(data_output_stream.into())
    }

    async fn write_byte(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, byte: i32) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::write({this:?}, {byte})");

        let mut buffer = jvm.instantiate_array("B", 1).await?;
        jvm.store_array(&mut buffer, 0, [byte as i8]).await?;

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (buffer, 0, 1)).await?;

        Ok(1)
    }

    async fn read_byte(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::read({this:?})");

        let buffer = jvm.instantiate_array("B", 1).await?;
        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let read: i32 = jvm.invoke_virtual(&raf, "read", "([BII)I", (buffer.clone(), 0, 1)).await?;
        if read <= 0 {
            return Ok(-1);
        }

        let mut value = [0u8; 1];
        jvm.array_raw_buffer(&buffer).await?.read(0, &mut value)?;

        Ok(value[0] as i32)
    }

    async fn tell(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("org.kwis.msp.io.File::tell({this:?})");

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let position: i64 = jvm.invoke_virtual(&raf, "getFilePointer", "()J", ()).await?;

        Ok(position as i32)
    }
}

#[cfg(test)]
mod test {
    use alloc::boxed::Box;

    use java_runtime::classes::java::{
        io::{DataOutputStream, OutputStream},
        lang::String,
    };
    use jvm::{ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;
    use wie_util::Result;

    use crate::{classes::net::wie::WIPIFileOutputStream, get_protos};

    use super::{File, Mode};

    #[test]
    fn test_byte_io_tell_and_output_streams() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), [WIPIFileOutputStream::as_proto()].into()]),
            |jvm| async move {
                let filename: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "byte-io.bin").await?.into();
                let file: ClassInstanceRef<File> = jvm
                    .new_class("org/kwis/msp/io/File", "(Ljava/lang/String;I)V", (filename, Mode::WRITE_TRUNC as i32))
                    .await?
                    .into();

                let initial: i32 = jvm.invoke_virtual(&file, "tell", "()I", ()).await?;
                assert_eq!(initial, 0);

                let written: i32 = jvm.invoke_virtual(&file, "write", "(I)I", (0xfe,)).await?;
                assert_eq!(written, 1);
                let after_write: i32 = jvm.invoke_virtual(&file, "tell", "()I", ()).await?;
                assert_eq!(after_write, 1);

                let _: () = jvm.invoke_virtual(&file, "seek", "(I)V", (0,)).await?;
                let value: i32 = jvm.invoke_virtual(&file, "read", "()I", ()).await?;
                let eof: i32 = jvm.invoke_virtual(&file, "read", "()I", ()).await?;
                assert_eq!(value, 0xfe);
                assert_eq!(eof, -1);

                let _: () = jvm.invoke_virtual(&file, "seek", "(I)V", (1,)).await?;
                let output: ClassInstanceRef<OutputStream> = jvm.invoke_virtual(&file, "openOutputStream", "()Ljava/io/OutputStream;", ()).await?;
                let _: () = jvm.invoke_virtual(&output, "write", "(I)V", (0xab,)).await?;
                let after_output_write: i32 = jvm.invoke_virtual(&file, "tell", "()I", ()).await?;
                assert_eq!(after_output_write, 2);

                let second_open: JvmResult<ClassInstanceRef<DataOutputStream>> = jvm
                    .invoke_virtual(&file, "openDataOutputStream", "()Ljava/io/DataOutputStream;", ())
                    .await;
                let Err(JavaError::JavaException(exception)) = second_open else {
                    panic!("concurrent data output stream opened");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let _: () = jvm.invoke_virtual(&output, "close", "()V", ()).await?;
                let closed_write: JvmResult<()> = jvm.invoke_virtual(&output, "write", "(I)V", (0xff,)).await;
                let Err(JavaError::JavaException(exception)) = closed_write else {
                    panic!("closed output stream accepted a write");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let data_output: ClassInstanceRef<DataOutputStream> = jvm
                    .invoke_virtual(&file, "openDataOutputStream", "()Ljava/io/DataOutputStream;", ())
                    .await?;
                let _: () = jvm.invoke_virtual(&output, "close", "()V", ()).await?;

                let second_open: JvmResult<ClassInstanceRef<OutputStream>> =
                    jvm.invoke_virtual(&file, "openOutputStream", "()Ljava/io/OutputStream;", ()).await;
                let Err(JavaError::JavaException(exception)) = second_open else {
                    panic!("closed old stream released the active data output stream");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let _: () = jvm.invoke_virtual(&data_output, "writeByte", "(I)V", (0xcd,)).await?;
                let after_data_output_write: i32 = jvm.invoke_virtual(&file, "tell", "()I", ()).await?;
                assert_eq!(after_data_output_write, 3);
                let _: () = jvm.invoke_virtual(&data_output, "close", "()V", ()).await?;

                let reopened: ClassInstanceRef<OutputStream> = jvm.invoke_virtual(&file, "openOutputStream", "()Ljava/io/OutputStream;", ()).await?;
                let _: () = jvm.invoke_virtual(&reopened, "close", "()V", ()).await?;

                let _: () = jvm.invoke_virtual(&file, "seek", "(I)V", (0,)).await?;
                let bytes = jvm.instantiate_array("B", 3).await?;
                let read: i32 = jvm.invoke_virtual(&file, "read", "([B)I", (bytes.clone(),)).await?;
                assert_eq!(read, 3);
                assert_eq!(jvm.load_array::<i8>(&bytes, 0, 3).await?, [-2i8, -85, -51]);

                Ok(())
            },
        )
    }

    #[test]
    fn test_output_stream_rejects_read_only_and_closed_file() -> Result<()> {
        run_jvm_test(
            Box::new([get_protos().into(), [WIPIFileOutputStream::as_proto()].into()]),
            |jvm| async move {
                let filename: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "read-only.bin").await?.into();
                let writable: ClassInstanceRef<File> = jvm
                    .new_class(
                        "org/kwis/msp/io/File",
                        "(Ljava/lang/String;I)V",
                        (filename.clone(), Mode::WRITE_TRUNC as i32),
                    )
                    .await?
                    .into();
                let _: i32 = jvm.invoke_virtual(&writable, "write", "(I)I", (1,)).await?;
                let _: () = jvm.invoke_virtual(&writable, "close", "()V", ()).await?;

                let read_only: ClassInstanceRef<File> = jvm
                    .new_class("org/kwis/msp/io/File", "(Ljava/lang/String;I)V", (filename, Mode::READ_ONLY as i32))
                    .await?
                    .into();
                let output: JvmResult<ClassInstanceRef<OutputStream>> =
                    jvm.invoke_virtual(&read_only, "openOutputStream", "()Ljava/io/OutputStream;", ()).await;
                let Err(JavaError::JavaException(exception)) = output else {
                    panic!("read-only file opened an output stream");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let data_output: JvmResult<ClassInstanceRef<DataOutputStream>> = jvm
                    .invoke_virtual(&read_only, "openDataOutputStream", "()Ljava/io/DataOutputStream;", ())
                    .await;
                let Err(JavaError::JavaException(exception)) = data_output else {
                    panic!("read-only file opened a data output stream");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let filename: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "closed.bin").await?.into();
                let closed: ClassInstanceRef<File> = jvm
                    .new_class("org/kwis/msp/io/File", "(Ljava/lang/String;I)V", (filename, Mode::WRITE_TRUNC as i32))
                    .await?
                    .into();
                let _: () = jvm.invoke_virtual(&closed, "close", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&closed, "close", "()V", ()).await?;

                let output: JvmResult<ClassInstanceRef<OutputStream>> =
                    jvm.invoke_virtual(&closed, "openOutputStream", "()Ljava/io/OutputStream;", ()).await;
                let Err(JavaError::JavaException(exception)) = output else {
                    panic!("closed file opened an output stream");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                Ok(())
            },
        )
    }
}
