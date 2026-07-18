use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::{io::InputStream, lang::String};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::xce::io::x_file::{FILE_JAR, NORMAL, READ, READ_RESOURCE, READ_WRITE, SEEK_CUR, SEEK_SET, STDIN, STDSTREAM, XFile};

// class com.xce.io.FileInputStream
pub struct FileInputStream;

impl FileInputStream {
    pub fn as_proto() -> WieJavaClassProto {
        let public = MethodAccessFlags::PUBLIC;

        WieJavaClassProto {
            name: "com/xce/io/FileInputStream",
            parent_class: Some("java/io/InputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init_with_fd, public),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, public),
                JavaMethodProto::new("<init>", "(Lcom/xce/io/XFile;)V", Self::init_with_file, public),
                JavaMethodProto::new("available", "()I", Self::available, public),
                JavaMethodProto::new("close", "()V", Self::close, public),
                JavaMethodProto::new("mark", "(I)V", Self::mark, public),
                JavaMethodProto::new("markSupported", "()Z", Self::mark_supported, public),
                JavaMethodProto::new("read", "()I", Self::read_byte, public),
                JavaMethodProto::new("read", "([B)I", Self::read_array_full, public),
                JavaMethodProto::new("read", "([BII)I", Self::read_array, public),
                JavaMethodProto::new("reset", "()V", Self::reset, public),
                JavaMethodProto::new("skip", "(J)J", Self::skip, public),
            ],
            fields: vec![
                JavaFieldProto::new("file", "Lcom/xce/io/XFile;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("markPosition", "I", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("marked", "Z", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({this:?}, {name:?})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }

        let file = jvm.new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, READ)).await?;
        jvm.put_field(&mut this, "file", "Lcom/xce/io/XFile;", file).await?;

        Ok(())
    }

    async fn init_with_fd(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fd: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({this:?}, {fd})");

        let _: () = jvm.invoke_special(&this, "java/io/InputStream", "<init>", "()V", ()).await?;
        if fd != STDIN {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "fd must be STDIN").await);
        }

        let file = jvm.new_class("com/xce/io/XFile", "(I)V", (fd,)).await?;
        jvm.put_field(&mut this, "file", "Lcom/xce/io/XFile;", file).await?;

        Ok(())
    }

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

        let file_type: i32 = jvm.get_field(&file, "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "mode", "I").await?;
        let fd: i32 = jvm.get_field(&file, "fd", "I").await?;
        let readable = match file_type {
            STDSTREAM => fd == STDIN && mode == READ,
            NORMAL => mode == READ || mode == READ_WRITE,
            FILE_JAR => mode == READ_RESOURCE,
            _ => false,
        };
        if !readable {
            return Err(jvm.exception("java/io/IOException", "XFile is not open for reading").await);
        }
        jvm.put_field(&mut this, "file", "Lcom/xce/io/XFile;", file).await?;

        Ok(())
    }

    async fn available(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::available({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let available = jvm.invoke_virtual(&file, "available", "()I", ()).await?;

        Ok(available)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::close({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let _: () = jvm.invoke_virtual(&file, "close", "()V", ()).await?;

        Ok(())
    }

    async fn mark(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, read_limit: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::mark({this:?}, {read_limit})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let file_type: i32 = jvm.get_field(&file, "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "mode", "I").await?;
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            let stream: ClassInstanceRef<InputStream> = jvm.get_field(&file, "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&stream, "mark", "(I)V", (read_limit,)).await?;
            let position: i32 = jvm.get_field(&file, "offset", "I").await?;
            jvm.put_field(&mut this, "markPosition", "I", position).await?;
        } else {
            let position: i32 = jvm.invoke_virtual(&file, "seek", "(II)I", (0, SEEK_CUR)).await?;
            jvm.put_field(&mut this, "markPosition", "I", position).await?;
        }
        jvm.put_field(&mut this, "marked", "Z", true).await?;

        Ok(())
    }

    async fn mark_supported(_jvm: &Jvm, _context: &mut WieJvmContext, _this: ClassInstanceRef<Self>) -> JvmResult<bool> {
        Ok(true)
    }

    async fn read_byte(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::read({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let buffer: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 1).await?.into();
        let read: i32 = jvm.invoke_virtual(&file, "read", "([BII)I", (buffer.clone(), 0, 1)).await?;
        if read <= 0 {
            return Ok(-1);
        }
        let byte = jvm.load_array::<i8>(&buffer, 0, 1).await?;

        Ok(byte[0] as u8 as i32)
    }

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

        jvm.invoke_virtual(&this, "read", "([BII)I", (buffer, 0, length)).await
    }

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

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let read: i32 = jvm.invoke_virtual(&file, "read", "([BII)I", (buf, offset, length)).await?;

        Ok(if read == 0 { -1 } else { read })
    }

    async fn reset(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::reset({this:?})");

        let marked: bool = jvm.get_field(&this, "marked", "Z").await?;
        if !marked {
            return Err(jvm.exception("java/io/IOException", "Stream has not been marked").await);
        }

        let mut file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let file_type: i32 = jvm.get_field(&file, "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "mode", "I").await?;
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            let stream: ClassInstanceRef<InputStream> = jvm.get_field(&file, "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&stream, "reset", "()V", ()).await?;
            let position: i32 = jvm.get_field(&this, "markPosition", "I").await?;
            jvm.put_field(&mut file, "offset", "I", position).await?;
        } else {
            let position: i32 = jvm.get_field(&this, "markPosition", "I").await?;
            let _: i32 = jvm.invoke_virtual(&file, "seek", "(II)I", (position, SEEK_SET)).await?;
        }

        Ok(())
    }

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
            let read: i32 = jvm.invoke_virtual(&this, "read", "([BII)I", (scratch.clone(), 0, request)).await?;
            if read <= 0 {
                break;
            }
            remaining -= read as i64;
        }

        Ok(count - remaining)
    }
}
