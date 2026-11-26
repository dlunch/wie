use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{io::InputStream, lang::String};

use wie_jvm_support::WieJvmContext;

use crate::classes::com::xce::io::x_file::{FILE_JAR, NORMAL, READ, READ_RESOURCE, READ_WRITE, SEEK_CUR, SEEK_SET, STDIN, STDSTREAM, XFile};

// class com.xce.io.FileInputStream
#[java_class(name = "com/xce/io/FileInputStream", extends = "java/io/InputStream", public)]
pub struct FileInputStream {
    #[field(private)]
    file: ClassInstanceRef<XFile>,
    #[field(private)]
    markPosition: i32,
    #[field(private)]
    marked: bool,
}

#[java_methods]
impl FileInputStream {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }

        let file = jvm.new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, READ)).await?;
        jvm.put_field(&mut this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;", file)
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_fd(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fd: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({this:?}, {fd})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        if fd != STDIN {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "fd must be STDIN").await);
        }

        let file = jvm.new_class("com/xce/io/XFile", "(I)V", (fd,)).await?;
        jvm.put_field(&mut this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;", file)
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_file(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        file: ClassInstanceRef<XFile>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({this:?}, {file:?})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        if file.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "file is null").await);
        }

        let file_type: i32 = jvm.get_field(&file, "com/xce/io/XFile", "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "com/xce/io/XFile", "mode", "I").await?;
        let fd: i32 = jvm.get_field(&file, "com/xce/io/XFile", "fd", "I").await?;
        let readable = match file_type {
            STDSTREAM => fd == STDIN && mode == READ,
            NORMAL => mode == READ || mode == READ_WRITE,
            FILE_JAR => mode == READ_RESOURCE,
            _ => false,
        };
        if !readable {
            return Err(jvm.exception("java/io/IOException", "XFile is not open for reading").await);
        }
        jvm.put_field(&mut this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;", file)
            .await?;

        Ok(())
    }

    #[method(public)]
    async fn available(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::available({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let available = jvm.invoke_virtual(&file, "com/xce/io/XFile", "available", "()I", ()).await?;

        Ok(available)
    }

    #[method(public)]
    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::close({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let _: () = jvm.invoke_virtual(&file, "com/xce/io/XFile", "close", "()V", ()).await?;

        Ok(())
    }

    #[method(public)]
    async fn mark(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, read_limit: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::mark({this:?}, {read_limit})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let file_type: i32 = jvm.get_field(&file, "com/xce/io/XFile", "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "com/xce/io/XFile", "mode", "I").await?;
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            let stream: ClassInstanceRef<InputStream> = jvm.get_field(&file, "com/xce/io/XFile", "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&stream, "java/io/InputStream", "mark", "(I)V", (read_limit,)).await?;
            let position: i32 = jvm.get_field(&file, "com/xce/io/XFile", "offset", "I").await?;
            jvm.put_field(&mut this, "com/xce/io/FileInputStream", "markPosition", "I", position)
                .await?;
        } else {
            let position: i32 = jvm.invoke_virtual(&file, "com/xce/io/XFile", "seek", "(II)I", (0, SEEK_CUR)).await?;
            jvm.put_field(&mut this, "com/xce/io/FileInputStream", "markPosition", "I", position)
                .await?;
        }
        jvm.put_field(&mut this, "com/xce/io/FileInputStream", "marked", "Z", true).await?;

        Ok(())
    }

    #[method(public)]
    async fn mark_supported(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        Ok(true)
    }

    #[method(name = "read", public)]
    async fn read_byte(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::read({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let buffer: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 1).await?.into();
        let read: i32 = jvm
            .invoke_virtual(&file, "com/xce/io/XFile", "read", "([BII)I", (buffer.clone(), 0, 1))
            .await?;
        if read <= 0 {
            return Ok(-1);
        }
        let byte = jvm.load_array::<i8>(&buffer, 0, 1).await?;

        Ok(byte[0] as u8 as i32)
    }

    #[method(name = "read", public)]
    async fn read_array_full(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::read({this:?}, {buffer:?})");

        if buffer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        }
        let length = jvm.array_length(&buffer).await? as i32;

        jvm.invoke_virtual(&this, "com/xce/io/FileInputStream", "read", "([BII)I", (buffer, 0, length))
            .await
    }

    #[method(name = "read", public)]
    async fn read_array(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buf: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::read({this:?}, {buf:?}, {offset}, {length})");

        if buf.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        }
        let array_length = jvm.array_length(&buf).await? as i32;
        if offset < 0 || length < 0 || offset > array_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(0);
        }

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let read: i32 = jvm
            .invoke_virtual(&file, "com/xce/io/XFile", "read", "([BII)I", (buf, offset, length))
            .await?;

        Ok(if read == 0 { -1 } else { read })
    }

    #[method(public)]
    async fn reset(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::reset({this:?})");

        let marked: bool = jvm.get_field(&this, "com/xce/io/FileInputStream", "marked", "Z").await?;
        if !marked {
            return Err(jvm.exception("java/io/IOException", "Stream has not been marked").await);
        }

        let mut file: ClassInstanceRef<XFile> = jvm.get_field(&this, "com/xce/io/FileInputStream", "file", "Lcom/xce/io/XFile;").await?;
        let file_type: i32 = jvm.get_field(&file, "com/xce/io/XFile", "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "com/xce/io/XFile", "mode", "I").await?;
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            let stream: ClassInstanceRef<InputStream> = jvm.get_field(&file, "com/xce/io/XFile", "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&stream, "java/io/InputStream", "reset", "()V", ()).await?;
            let position: i32 = jvm.get_field(&this, "com/xce/io/FileInputStream", "markPosition", "I").await?;
            jvm.put_field(&mut file, "com/xce/io/XFile", "offset", "I", position).await?;
        } else {
            let position: i32 = jvm.get_field(&this, "com/xce/io/FileInputStream", "markPosition", "I").await?;
            let _: i32 = jvm
                .invoke_virtual(&file, "com/xce/io/XFile", "seek", "(II)I", (position, SEEK_SET))
                .await?;
        }

        Ok(())
    }

    #[method(public)]
    async fn skip(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, count: i64) -> JvmResult<i64> {
        tracing::debug!("com.xce.io.FileInputStream::skip({this:?}, {count})");

        if count <= 0 {
            return Ok(0);
        }

        let scratch_size = count.min(4096) as usize;
        let scratch: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", scratch_size).await?.into();
        let mut remaining = count;
        while remaining > 0 {
            let request = remaining.min(scratch_size as i64) as i32;
            let read: i32 = jvm
                .invoke_virtual(&this, "com/xce/io/FileInputStream", "read", "([BII)I", (scratch.clone(), 0, request))
                .await?;
            if read <= 0 {
                break;
            }
            remaining -= read as i64;
        }

        Ok(count - remaining)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use jvm::{Array, ClassInstanceRef, runtime::JavaLangString};
    use rustjava_runtime::classes::java::lang::String;
    use test_utils::run_jvm_test;

    use crate::classes::com::xce::io::{FileInputStream, FileOutputStream, XFile};

    #[test]
    fn stream_overloads_preserve_mark_skip_and_eof_behavior() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "stream-overloads.dat").await?.into();
                let output: ClassInstanceRef<FileOutputStream> = jvm
                    .new_class("com/xce/io/FileOutputStream", "(Ljava/lang/String;Z)V", (name.clone(), true))
                    .await?
                    .into();
                let mut source = jvm.instantiate_array("B", 4).await?;
                jvm.store_array(&mut source, 0, [1_i8, 2, 3, 4]).await?;
                let _: () = jvm
                    .invoke_virtual(&output, "com/xce/io/FileOutputStream", "write", "([BII)V", (source, 1, 2))
                    .await?;
                let _: () = jvm.invoke_virtual(&output, "com/xce/io/FileOutputStream", "flush", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&output, "com/xce/io/FileOutputStream", "close", "()V", ()).await?;

                let append_output: ClassInstanceRef<FileOutputStream> = jvm
                    .new_class("com/xce/io/FileOutputStream", "(Ljava/lang/String;Z)V", (name.clone(), false))
                    .await?
                    .into();
                let _: () = jvm
                    .invoke_virtual(&append_output, "com/xce/io/FileOutputStream", "write", "(I)V", (4,))
                    .await?;
                let _: () = jvm
                    .invoke_virtual(&append_output, "com/xce/io/FileOutputStream", "close", "()V", ())
                    .await?;

                let input: ClassInstanceRef<FileInputStream> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Ljava/lang/String;)V", (name,))
                    .await?
                    .into();
                assert!(
                    jvm.invoke_virtual::<_, bool>(&input, "com/xce/io/FileInputStream", "markSupported", "()Z", ())
                        .await?
                );
                let _: () = jvm.invoke_virtual(&input, "com/xce/io/FileInputStream", "mark", "(I)V", (8,)).await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "read", "()I", ())
                        .await?,
                    2
                );
                let _: () = jvm.invoke_virtual(&input, "com/xce/io/FileInputStream", "reset", "()V", ()).await?;
                assert_eq!(
                    jvm.invoke_virtual::<_, i64>(&input, "com/xce/io/FileInputStream", "skip", "(J)J", (1_i64,))
                        .await?,
                    1
                );

                let target: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 2).await?.into();
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "read", "([B)I", (target.clone(),))
                        .await?,
                    2
                );
                assert_eq!(jvm.load_array::<i8>(&target, 0, 2).await?, [3, 4]);
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "read", "()I", ())
                        .await?,
                    -1
                );

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn append_mode_creates_a_missing_file() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "append-new.dat").await?.into();
                let output: ClassInstanceRef<FileOutputStream> = jvm
                    .new_class("com/xce/io/FileOutputStream", "(Ljava/lang/String;Z)V", (name.clone(), false))
                    .await?
                    .into();
                let _: () = jvm
                    .invoke_virtual(&output, "com/xce/io/FileOutputStream", "write", "(I)V", (0xAB,))
                    .await?;
                let _: () = jvm.invoke_virtual(&output, "com/xce/io/FileOutputStream", "close", "()V", ()).await?;

                let input: ClassInstanceRef<FileInputStream> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Ljava/lang/String;)V", (name,))
                    .await?
                    .into();
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "read", "()I", ())
                        .await?,
                    0xAB
                );

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn standard_streams_have_neutral_input_and_non_owning_output_close() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let input: ClassInstanceRef<FileInputStream> = jvm.new_class("com/xce/io/FileInputStream", "(I)V", (0,)).await?.into();
                assert!(
                    jvm.invoke_virtual::<_, bool>(&input, "com/xce/io/FileInputStream", "markSupported", "()Z", ())
                        .await?
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "available", "()I", ())
                        .await?,
                    0
                );
                assert_eq!(
                    jvm.invoke_virtual::<_, i32>(&input, "com/xce/io/FileInputStream", "read", "()I", ())
                        .await?,
                    -1
                );
                let _: () = jvm.invoke_virtual(&input, "com/xce/io/FileInputStream", "mark", "(I)V", (1,)).await?;
                let _: () = jvm.invoke_virtual(&input, "com/xce/io/FileInputStream", "reset", "()V", ()).await?;

                let first: ClassInstanceRef<FileOutputStream> = jvm.new_class("com/xce/io/FileOutputStream", "(I)V", (1,)).await?.into();
                let _: () = jvm.invoke_virtual(&first, "com/xce/io/FileOutputStream", "write", "(I)V", (1,)).await?;
                let _: () = jvm.invoke_virtual(&first, "com/xce/io/FileOutputStream", "close", "()V", ()).await?;
                let second: ClassInstanceRef<FileOutputStream> = jvm.new_class("com/xce/io/FileOutputStream", "(I)V", (1,)).await?.into();
                let _: () = jvm.invoke_virtual(&second, "com/xce/io/FileOutputStream", "write", "(I)V", (2,)).await?;
                let _: () = jvm.invoke_virtual(&second, "com/xce/io/FileOutputStream", "close", "()V", ()).await?;

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
