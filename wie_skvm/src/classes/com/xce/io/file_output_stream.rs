use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::xce::io::x_file::{NORMAL, READ_WRITE, SEEK_END, SEEK_SET, STDERR, STDOUT, STDSTREAM, WRITE, XFile};

// class com.xce.io.FileOutputStream
pub struct FileOutputStream;

impl FileOutputStream {
    pub fn as_proto() -> WieJavaClassProto {
        let public = MethodAccessFlags::PUBLIC;

        WieJavaClassProto {
            name: "com/xce/io/FileOutputStream",
            parent_class: Some("java/io/OutputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(I)V", Self::init_with_fd, public),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, public),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Z)V", Self::init_with_truncate, public),
                JavaMethodProto::new("<init>", "(Lcom/xce/io/XFile;)V", Self::init_with_file, public),
                JavaMethodProto::new("close", "()V", Self::close, public),
                JavaMethodProto::new("flush", "()V", Self::flush, public),
                JavaMethodProto::new("write", "([BII)V", Self::write_array, public),
                JavaMethodProto::new("write", "(I)V", Self::write, public),
            ],
            fields: vec![JavaFieldProto::new("file", "Lcom/xce/io/XFile;", FieldAccessFlags::PROTECTED)],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::<init>({this:?}, {name:?})");

        let _: () = jvm
            .invoke_special(&this, "com/xce/io/FileOutputStream", "<init>", "(Ljava/lang/String;Z)V", (name, true))
            .await?;

        Ok(())
    }

    async fn init_with_truncate(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
        truncate: bool,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::<init>({this:?}, {name:?}, {truncate})");

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;
        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }

        let existed = if truncate {
            false
        } else {
            jvm.invoke_static("com/xce/io/XFile", "exists", "(Ljava/lang/String;)Z", (name.clone(),))
                .await?
        };
        let file: ClassInstanceRef<XFile> = jvm.new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, WRITE)).await?.into();
        if truncate {
            let raf = XFile::raf(jvm, file.clone()).await?;
            let _: () = jvm.invoke_virtual(&raf, "setLength", "(J)V", (0_i64,)).await?;
        }
        let whence = if !truncate && existed { SEEK_END } else { SEEK_SET };
        let _: i32 = jvm.invoke_virtual(&file, "seek", "(II)I", (0, whence)).await?;
        jvm.put_field(&mut this, "file", "Lcom/xce/io/XFile;", file).await?;

        Ok(())
    }

    async fn init_with_fd(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fd: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::<init>({this:?}, {fd})");

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;
        if fd != STDOUT && fd != STDERR {
            return Err(jvm.exception("java/lang/IllegalArgumentException", "fd must be STDOUT or STDERR").await);
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
        tracing::debug!("com.xce.io.FileOutputStream::<init>({file:?})");

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;
        if file.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "file is null").await);
        }

        let file_type: i32 = jvm.get_field(&file, "type", "I").await?;
        let mode: i32 = jvm.get_field(&file, "mode", "I").await?;
        let fd: i32 = jvm.get_field(&file, "fd", "I").await?;
        let writable = match file_type {
            STDSTREAM => (fd == STDOUT || fd == STDERR) && mode == WRITE,
            NORMAL => mode == WRITE || mode == READ_WRITE,
            _ => false,
        };
        if !writable {
            return Err(jvm.exception("java/io/IOException", "XFile is not open for writing").await);
        }
        jvm.put_field(&mut this, "file", "Lcom/xce/io/XFile;", file).await?;

        Ok(())
    }

    async fn write(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, byte: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::write({this:?}, {byte:?})");

        let mut buffer = jvm.instantiate_array("B", 1).await?;
        jvm.store_array(&mut buffer, 0, [byte as i8]).await?;
        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let _: i32 = jvm.invoke_virtual(&file, "write", "([BII)I", (buffer, 0, 1)).await?;

        Ok(())
    }

    async fn write_array(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        buffer: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::write({this:?}, {buffer:?}, {offset}, {length})");

        if buffer.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        }
        let array_length = jvm.array_length(&buffer).await? as i32;
        if offset < 0 || length < 0 || offset > array_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(());
        }

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let _: i32 = jvm.invoke_virtual(&file, "write", "([BII)I", (buffer, offset, length)).await?;

        Ok(())
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::close({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let _: () = jvm.invoke_virtual(&file, "close", "()V", ()).await?;

        Ok(())
    }

    async fn flush(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::flush({this:?})");

        let file: ClassInstanceRef<XFile> = jvm.get_field(&this, "file", "Lcom/xce/io/XFile;").await?;
        let _: () = jvm.invoke_virtual(&file, "flush", "()V", ()).await?;

        Ok(())
    }
}
