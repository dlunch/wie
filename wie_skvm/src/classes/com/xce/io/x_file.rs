use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{ClassAccessFlags, FieldAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::{
    io::{FileDescriptor, InputStream, OutputStream, RandomAccessFile},
    lang::String,
};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

pub(super) const STDSTREAM: i32 = 0;
pub(super) const NORMAL: i32 = 1;
const DIRECTORY: i32 = 2;
pub(super) const FILE_JAR: i32 = 3;

pub(super) const STDIN: i32 = 0;
pub(super) const STDOUT: i32 = 1;
pub(super) const STDERR: i32 = 2;

pub(super) const SEEK_SET: i32 = 0;
pub(super) const SEEK_CUR: i32 = 1;
pub(super) const SEEK_END: i32 = 2;

pub(super) const READ: i32 = 1;
pub(super) const WRITE: i32 = 2;
pub(super) const READ_WRITE: i32 = 3;
const READ_DIRECTORY: i32 = 4;
pub(super) const READ_RESOURCE: i32 = 8;

// class com.xce.io.XFile
pub struct XFile;

impl XFile {
    pub fn as_proto() -> WieJavaClassProto {
        let public = MethodAccessFlags::PUBLIC;
        let public_static = MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC;
        let public_static_final = FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC | FieldAccessFlags::FINAL;

        WieJavaClassProto {
            name: "com/xce/io/XFile",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC),
                JavaMethodProto::new("<init>", "(I)V", Self::init_with_fd, public),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, public),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;Ljava/lang/String;)V", Self::init_with_jar, public),
                JavaMethodProto::new("available", "()I", Self::available, public),
                JavaMethodProto::new("close", "()V", Self::close, public),
                JavaMethodProto::new("flush", "()V", Self::flush, public),
                JavaMethodProto::new("read", "([BII)I", Self::read, public),
                JavaMethodProto::new("readdir", "()Ljava/lang/String;", Self::read_dir, public),
                JavaMethodProto::new("seek", "(II)I", Self::seek, public),
                JavaMethodProto::new("write", "([BII)I", Self::write, public),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, public_static),
                JavaMethodProto::new("filesize", "(Ljava/lang/String;)I", Self::filesize, public_static),
                JavaMethodProto::new("fsavail", "()I", Self::fs_available, public_static),
                JavaMethodProto::new("fsused", "()I", Self::fs_used, public_static),
                JavaMethodProto::new("mkdir", "(Ljava/lang/String;)V", Self::make_dir, public_static),
                JavaMethodProto::new("rmdir", "(Ljava/lang/String;)V", Self::remove_dir, public_static),
                JavaMethodProto::new("rmrdir", "(Ljava/lang/String;)V", Self::remove_dir_recursive, public_static),
                JavaMethodProto::new("unlink", "(Ljava/lang/String;)I", Self::unlink, public_static),
            ],
            fields: vec![
                JavaFieldProto::new("STDSTREAM", "I", public_static_final),
                JavaFieldProto::new("NORMAL", "I", public_static_final),
                JavaFieldProto::new("DIRECTORY", "I", public_static_final),
                JavaFieldProto::new("FILE_JAR", "I", public_static_final),
                JavaFieldProto::new("STDIN", "I", public_static_final),
                JavaFieldProto::new("STDOUT", "I", public_static_final),
                JavaFieldProto::new("STDERR", "I", public_static_final),
                JavaFieldProto::new("SEEK_SET", "I", public_static_final),
                JavaFieldProto::new("SEEK_CUR", "I", public_static_final),
                JavaFieldProto::new("SEEK_END", "I", public_static_final),
                JavaFieldProto::new("READ", "I", public_static_final),
                JavaFieldProto::new("WRITE", "I", public_static_final),
                JavaFieldProto::new("READ_WRITE", "I", public_static_final),
                JavaFieldProto::new("READ_DIRECTORY", "I", public_static_final),
                JavaFieldProto::new("READ_RESOURCE", "I", public_static_final),
                JavaFieldProto::new("type", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("mode", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("fd", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("offset", "I", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("buf", "[B", FieldAccessFlags::PROTECTED),
                JavaFieldProto::new("is", "Ljava/io/InputStream;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("os", "Ljava/io/OutputStream;", FieldAccessFlags::PRIVATE),
                JavaFieldProto::new("raf", "Ljava/io/RandomAccessFile;", FieldAccessFlags::PRIVATE),
            ],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn cl_init(jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::<clinit>()");

        jvm.put_static_field("com/xce/io/XFile", "STDSTREAM", "I", STDSTREAM).await?;
        jvm.put_static_field("com/xce/io/XFile", "NORMAL", "I", NORMAL).await?;
        jvm.put_static_field("com/xce/io/XFile", "DIRECTORY", "I", DIRECTORY).await?;
        jvm.put_static_field("com/xce/io/XFile", "FILE_JAR", "I", FILE_JAR).await?;
        jvm.put_static_field("com/xce/io/XFile", "STDIN", "I", STDIN).await?;
        jvm.put_static_field("com/xce/io/XFile", "STDOUT", "I", STDOUT).await?;
        jvm.put_static_field("com/xce/io/XFile", "STDERR", "I", STDERR).await?;
        jvm.put_static_field("com/xce/io/XFile", "SEEK_SET", "I", SEEK_SET).await?;
        jvm.put_static_field("com/xce/io/XFile", "SEEK_CUR", "I", SEEK_CUR).await?;
        jvm.put_static_field("com/xce/io/XFile", "SEEK_END", "I", SEEK_END).await?;
        jvm.put_static_field("com/xce/io/XFile", "READ", "I", READ).await?;
        jvm.put_static_field("com/xce/io/XFile", "WRITE", "I", WRITE).await?;
        jvm.put_static_field("com/xce/io/XFile", "READ_WRITE", "I", READ_WRITE).await?;
        jvm.put_static_field("com/xce/io/XFile", "READ_DIRECTORY", "I", READ_DIRECTORY).await?;
        jvm.put_static_field("com/xce/io/XFile", "READ_RESOURCE", "I", READ_RESOURCE).await?;

        Ok(())
    }

    async fn init_with_fd(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, fd: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::<init>({this:?}, {fd})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        if fd != STDIN && fd != STDOUT && fd != STDERR {
            return Err(jvm
                .exception("java/lang/IllegalArgumentException", "Invalid standard stream descriptor")
                .await);
        }

        jvm.put_field(&mut this, "type", "I", STDSTREAM).await?;
        jvm.put_field(&mut this, "mode", "I", if fd == STDIN { READ } else { WRITE }).await?;
        jvm.put_field(&mut this, "fd", "I", fd).await?;
        jvm.put_field(&mut this, "offset", "I", 0).await?;

        if fd == STDIN {
            let empty: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 0).await?.into();
            let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (empty,)).await?;
            jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", stream).await?;
        } else {
            let field_name = if fd == STDOUT { "out" } else { "err" };
            let descriptor: ClassInstanceRef<FileDescriptor> = jvm
                .get_static_field("java/io/FileDescriptor", field_name, "Ljava/io/FileDescriptor;")
                .await?;
            if descriptor.is_null() {
                return Err(jvm.exception("java/io/IOException", "Standard stream is not supported").await);
            }
            let stream = jvm
                .new_class("java/io/FileOutputStream", "(Ljava/io/FileDescriptor;)V", (descriptor,))
                .await?;
            jvm.put_field(&mut this, "os", "Ljava/io/OutputStream;", stream).await?;
        }

        Ok(())
    }

    async fn init(
        jvm: &Jvm,
        context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::<init>({this:?}, {name:?}, {mode:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }

        jvm.put_field(&mut this, "mode", "I", mode).await?;
        jvm.put_field(&mut this, "offset", "I", 0).await?;

        if mode == READ_RESOURCE {
            let class = jvm.invoke_virtual(&this, "getClass", "()Ljava/lang/Class;", ()).await?;
            let resource_stream: ClassInstanceRef<InputStream> = jvm
                .invoke_virtual(&class, "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;", (name,))
                .await?;
            if resource_stream.is_null() {
                return Err(jvm.exception("java/io/IOException", "Resource not found").await);
            }

            jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", resource_stream).await?;
            jvm.put_field(&mut this, "type", "I", FILE_JAR).await?;
        } else {
            if mode == READ_DIRECTORY {
                return Err(jvm.exception("java/io/IOException", "Directory reads are not supported").await);
            }
            if mode != READ && mode != WRITE && mode != READ_WRITE {
                return Err(jvm.exception("java/lang/IllegalArgumentException", "Invalid file mode").await);
            }

            let guest_path = JavaLangString::to_rust_string(jvm, &name).await?;
            if !context.system().filesystem().is_valid_path(&guest_path) {
                return Err(jvm.exception("java/io/IOException", "Invalid file path").await);
            }

            let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;

            let file_mode = if mode == READ { "r" } else { "rw" };
            let file_mode = JavaLangString::from_rust_string(jvm, file_mode).await?;

            let raf: ClassInstanceRef<RandomAccessFile> = jvm
                .new_class("java/io/RandomAccessFile", "(Ljava/io/File;Ljava/lang/String;)V", (file, file_mode))
                .await?
                .into();
            let descriptor: ClassInstanceRef<FileDescriptor> = jvm.invoke_virtual(&raf, "getFD", "()Ljava/io/FileDescriptor;", ()).await?;
            let fd: i32 = jvm.get_field(&descriptor, "fd", "I").await?;

            jvm.put_field(&mut this, "type", "I", NORMAL).await?;
            jvm.put_field(&mut this, "fd", "I", fd).await?;
            jvm.put_field(&mut this, "raf", "Ljava/io/RandomAccessFile;", raf).await?;
        }

        Ok(())
    }

    async fn init_with_jar(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        jar_file: ClassInstanceRef<String>,
        name: ClassInstanceRef<String>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::<init>({this:?}, {jar_file:?}, {name:?})");

        if jar_file.is_null() || name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "jarfile and name must not be null").await);
        }

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;
        tracing::warn!("unsupported com.xce.io.XFile::<init>(jarfile, name)");
        Err(jvm.exception("java/io/IOException", "JAR file selection is not supported").await)
    }

    async fn exists(jvm: &Jvm, context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("com.xce.io.XFile::exists({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }
        let guest_path = JavaLangString::to_rust_string(jvm, &name).await?;
        if !context.system().filesystem().is_valid_path(&guest_path) {
            return Err(jvm.exception("java/io/IOException", "Invalid file path").await);
        }

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;

        Ok(exists)
    }

    async fn filesize(jvm: &Jvm, context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::filesize({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }
        let guest_path = JavaLangString::to_rust_string(jvm, &name).await?;
        if !context.system().filesystem().is_valid_path(&guest_path) {
            return Err(jvm.exception("java/io/IOException", "Invalid file path").await);
        }

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists: bool = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;
        if !exists {
            return Err(jvm.exception("java/io/IOException", "File not found").await);
        }
        let size: i64 = jvm.invoke_virtual(&file, "length", "()J", ()).await?;
        if size < 0 || size > i32::MAX as i64 {
            return Err(jvm.exception("java/io/IOException", "File is too large").await);
        }

        Ok(size as i32)
    }

    async fn unlink(jvm: &Jvm, context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::unlink({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "name is null").await);
        }
        let guest_path = JavaLangString::to_rust_string(jvm, &name).await?;
        if !context.system().filesystem().is_valid_path(&guest_path) {
            return Err(jvm.exception("java/io/IOException", "Invalid file path").await);
        }

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let deleted: bool = jvm.invoke_virtual(&file, "delete", "()Z", ()).await?;
        if !deleted {
            return Err(jvm.exception("java/io/IOException", "Unable to delete file").await);
        }

        Ok(0)
    }

    async fn available(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::available({this:?})");

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode != READ && mode != READ_WRITE && mode != READ_RESOURCE {
            return Err(jvm.exception("java/io/IOException", "File is not open for reading").await);
        }
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            if is.is_null() {
                return Err(jvm.exception("java/io/IOException", "File is not open for reading").await);
            }
            let available = jvm.invoke_virtual(&is, "available", "()I", ()).await?;

            Ok(available)
        } else {
            let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let file_length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;
            let file_pointer: i64 = jvm.invoke_virtual(&raf, "getFilePointer", "()J", ()).await?;

            Ok((file_length - file_pointer).clamp(0, i32::MAX as i64) as i32)
        }
    }

    async fn read(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::read({this:?}, {data:?}, {offset}, {length})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        }
        let array_length = jvm.array_length(&data).await? as i32;
        if offset < 0 || length < 0 || offset > array_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(0);
        }

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        let read = if file_type == STDSTREAM || mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            if is.is_null() {
                return Err(jvm.exception("java/io/IOException", "File is not open for reading").await);
            }

            jvm.invoke_virtual(&is, "read", "([BII)I", (data, offset, length)).await?
        } else {
            if mode != READ && mode != READ_WRITE {
                return Err(jvm.exception("java/io/IOException", "File is not open for reading").await);
            }
            let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;

            jvm.invoke_virtual(&raf, "read", "([BII)I", (data, offset, length)).await?
        };

        if read > 0 {
            let old_offset: i32 = jvm.get_field(&this, "offset", "I").await?;
            jvm.put_field(&mut this, "offset", "I", old_offset.saturating_add(read)).await?;
        }

        Ok(read)
    }

    async fn write(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::write({this:?}, {data:?}, {offset}, {length})");

        if data.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "buffer is null").await);
        }
        let array_length = jvm.array_length(&data).await? as i32;
        if offset < 0 || length < 0 || offset > array_length - length {
            return Err(jvm.exception("java/lang/IndexOutOfBoundsException", "Invalid offset or length").await);
        }
        if length == 0 {
            return Ok(0);
        }

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if file_type == STDSTREAM {
            let os: ClassInstanceRef<OutputStream> = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;
            if os.is_null() {
                return Err(jvm.exception("java/io/IOException", "File is not open for writing").await);
            }
            let _: () = jvm.invoke_virtual(&os, "write", "([BII)V", (data, offset, length)).await?;
        } else {
            if mode != WRITE && mode != READ_WRITE {
                return Err(jvm.exception("java/io/IOException", "File is not open for writing").await);
            }
            let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (data, offset, length)).await?;
        }

        let old_offset: i32 = jvm.get_field(&this, "offset", "I").await?;
        jvm.put_field(&mut this, "offset", "I", old_offset.saturating_add(length)).await?;

        Ok(length)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::close({this:?})");

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if file_type == STDSTREAM {
            return Ok(());
        } else if mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&is, "close", "()V", ()).await?;
        } else {
            let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let _: () = jvm.invoke_virtual(&raf, "close", "()V", ()).await?;
        }

        Ok(())
    }

    async fn flush(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::flush({this:?})");

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        if file_type == STDSTREAM {
            let os: ClassInstanceRef<OutputStream> = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;
            if os.is_null() {
                return Err(jvm.exception("java/io/IOException", "File is not open for writing").await);
            }
            let _: () = jvm.invoke_virtual(&os, "flush", "()V", ()).await?;
        }

        Ok(())
    }

    async fn seek(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, n: i32, whence: i32) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::seek({this:?}, {n}, {whence})");

        let file_type: i32 = jvm.get_field(&this, "type", "I").await?;
        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if file_type == STDSTREAM || mode == READ_RESOURCE {
            return Err(jvm.exception("java/io/IOException", "File is not seekable").await);
        }

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let new_pos = match whence {
            SEEK_SET => Some(n as i64),
            SEEK_CUR => {
                let current: i64 = jvm.invoke_virtual(&raf, "getFilePointer", "()J", ()).await?;
                current.checked_add(n as i64)
            }
            SEEK_END => {
                let length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;
                length.checked_add(n as i64)
            }
            _ => return Err(jvm.exception("java/io/IOException", "Invalid whence").await),
        };
        let Some(new_pos) = new_pos else {
            return Err(jvm.exception("java/io/IOException", "Seek position overflow").await);
        };
        if new_pos < 0 || new_pos > i32::MAX as i64 {
            return Err(jvm.exception("java/io/IOException", "Invalid seek position").await);
        }

        let _: () = jvm.invoke_virtual(&raf, "seek", "(J)V", (new_pos,)).await?;
        jvm.put_field(&mut this, "offset", "I", new_pos as i32).await?;

        Ok(new_pos as i32)
    }

    async fn read_dir(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<String>> {
        tracing::warn!("unsupported com.xce.io.XFile::readdir({this:?})");

        Err(jvm.exception("java/io/IOException", "Directory reads are not supported").await)
    }

    async fn make_dir(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("unsupported com.xce.io.XFile::mkdir({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "dirname is null").await);
        }
        Err(jvm.exception("java/io/IOException", "Directory creation is not supported").await)
    }

    async fn remove_dir(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("unsupported com.xce.io.XFile::rmdir({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "dirname is null").await);
        }
        Err(jvm.exception("java/io/IOException", "Directory removal is not supported").await)
    }

    async fn remove_dir_recursive(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("unsupported com.xce.io.XFile::rmrdir({name:?})");

        if name.is_null() {
            return Err(jvm.exception("java/lang/NullPointerException", "dirname is null").await);
        }
        Err(jvm.exception("java/io/IOException", "Recursive directory removal is not supported").await)
    }

    async fn fs_used(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::fsused()");

        Ok(0)
    }

    async fn fs_available(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::fsavail()");

        Ok(0)
    }

    pub async fn raf(jvm: &Jvm, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<RandomAccessFile>> {
        let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        if raf.is_null() {
            return Err(jvm.exception("java/io/IOException", "File has no random access handle").await);
        }

        Ok(raf)
    }
}

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;

    use java_runtime::classes::java::lang::String;
    use jvm::{Array, ClassInstanceRef, JavaError, Result as JvmResult, runtime::JavaLangString};
    use test_utils::run_jvm_test;

    use super::{FILE_JAR, READ_RESOURCE, XFile};
    use crate::classes::com::xce::io::{file_input_stream::FileInputStream, file_output_stream::FileOutputStream};

    #[test]
    fn xfile_write_read_and_seek_round_trip() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let read_write: i32 = jvm.get_static_field("com/xce/io/XFile", "READ_WRITE", "I").await?;
                let seek_set: i32 = jvm.get_static_field("com/xce/io/XFile", "SEEK_SET", "I").await?;
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "xfile-round-trip.dat").await?.into();
                let file: ClassInstanceRef<XFile> = jvm
                    .new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, read_write))
                    .await?
                    .into();

                let mut source = jvm.instantiate_array("B", 4).await?;
                jvm.store_array(&mut source, 0, [10_i8, 20, 30, 40]).await?;
                let written: i32 = jvm.invoke_virtual(&file, "write", "([BII)I", (source, 1, 2)).await?;
                assert_eq!(written, 2);

                let position: i32 = jvm.invoke_virtual(&file, "seek", "(II)I", (0, seek_set)).await?;
                assert_eq!(position, 0);

                let target: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 4).await?.into();
                let read: i32 = jvm.invoke_virtual(&file, "read", "([BII)I", (target.clone(), 1, 2)).await?;
                assert_eq!(read, 2);
                assert_eq!(jvm.load_array::<i8>(&target, 0, 4).await?, [0, 20, 30, 0]);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

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
                let _: () = jvm.invoke_virtual(&output, "write", "([BII)V", (source, 1, 2)).await?;
                let _: () = jvm.invoke_virtual(&output, "flush", "()V", ()).await?;
                let _: () = jvm.invoke_virtual(&output, "close", "()V", ()).await?;

                let append_output: ClassInstanceRef<FileOutputStream> = jvm
                    .new_class("com/xce/io/FileOutputStream", "(Ljava/lang/String;Z)V", (name.clone(), false))
                    .await?
                    .into();
                let _: () = jvm.invoke_virtual(&append_output, "write", "(I)V", (4,)).await?;
                let _: () = jvm.invoke_virtual(&append_output, "close", "()V", ()).await?;

                let input: ClassInstanceRef<FileInputStream> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Ljava/lang/String;)V", (name,))
                    .await?
                    .into();
                assert!(jvm.invoke_virtual::<_, bool>(&input, "markSupported", "()Z", ()).await?);
                let _: () = jvm.invoke_virtual(&input, "mark", "(I)V", (8,)).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, 2);
                let _: () = jvm.invoke_virtual(&input, "reset", "()V", ()).await?;
                assert_eq!(jvm.invoke_virtual::<_, i64>(&input, "skip", "(J)J", (1_i64,)).await?, 1);

                let target: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 2).await?.into();
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "([B)I", (target.clone(),)).await?, 2);
                assert_eq!(jvm.load_array::<i8>(&target, 0, 2).await?, [3, 4]);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, -1);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn file_array_errors_are_java_exceptions() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let read_write: i32 = jvm.get_static_field("com/xce/io/XFile", "READ_WRITE", "I").await?;
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "xfile-errors.dat").await?.into();
                let file: ClassInstanceRef<XFile> = jvm
                    .new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, read_write))
                    .await?
                    .into();
                let data: ClassInstanceRef<Array<i8>> = jvm.instantiate_array("B", 2).await?.into();

                let range_result: JvmResult<i32> = jvm.invoke_virtual(&file, "read", "([BII)I", (data, i32::MAX, 1)).await;
                let Err(JavaError::JavaException(exception)) = range_result else {
                    panic!("XFile.read accepted an invalid range");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/IndexOutOfBoundsException"));

                let null_data = ClassInstanceRef::<Array<i8>>::new(None);
                let null_result: JvmResult<i32> = jvm.invoke_virtual(&file, "write", "([BII)I", (null_data, 0, 1)).await;
                let Err(JavaError::JavaException(exception)) = null_result else {
                    panic!("XFile.write accepted null");
                };
                assert!(jvm.is_instance(&*exception, "java/lang/NullPointerException"));

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
                let _: () = jvm.invoke_virtual(&output, "write", "(I)V", (0xAB,)).await?;
                let _: () = jvm.invoke_virtual(&output, "close", "()V", ()).await?;

                let input: ClassInstanceRef<FileInputStream> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Ljava/lang/String;)V", (name,))
                    .await?
                    .into();
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, 0xAB);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn invalid_paths_and_unsupported_jar_selection_raise_io_exception() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let write: i32 = jvm.get_static_field("com/xce/io/XFile", "WRITE", "I").await?;
                let traversal: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "../escape.dat").await?.into();
                let open_result: JvmResult<ClassInstanceRef<XFile>> = jvm
                    .new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (traversal, write))
                    .await
                    .map(Into::into);
                let Err(JavaError::JavaException(exception)) = open_result else {
                    panic!("XFile accepted a traversal path");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let trailing: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "missing/").await?.into();
                let metadata_result: JvmResult<bool> = jvm
                    .invoke_static("com/xce/io/XFile", "exists", "(Ljava/lang/String;)Z", (trailing,))
                    .await;
                let Err(JavaError::JavaException(exception)) = metadata_result else {
                    panic!("XFile.exists accepted a trailing-slash path");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let jar: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "other.jar").await?.into();
                let entry: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "entry.dat").await?.into();
                let jar_result: JvmResult<ClassInstanceRef<XFile>> = jvm
                    .new_class("com/xce/io/XFile", "(Ljava/lang/String;Ljava/lang/String;)V", (jar, entry))
                    .await
                    .map(Into::into);
                let Err(JavaError::JavaException(exception)) = jar_result else {
                    panic!("XFile silently ignored the selected JAR");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

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
                assert!(jvm.invoke_virtual::<_, bool>(&input, "markSupported", "()Z", ()).await?);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "available", "()I", ()).await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, -1);
                let _: () = jvm.invoke_virtual(&input, "mark", "(I)V", (1,)).await?;
                let _: () = jvm.invoke_virtual(&input, "reset", "()V", ()).await?;

                let first: ClassInstanceRef<FileOutputStream> = jvm.new_class("com/xce/io/FileOutputStream", "(I)V", (1,)).await?.into();
                let _: () = jvm.invoke_virtual(&first, "write", "(I)V", (1,)).await?;
                let _: () = jvm.invoke_virtual(&first, "close", "()V", ()).await?;
                let second: ClassInstanceRef<FileOutputStream> = jvm.new_class("com/xce/io/FileOutputStream", "(I)V", (1,)).await?.into();
                let _: () = jvm.invoke_virtual(&second, "write", "(I)V", (2,)).await?;
                let _: () = jvm.invoke_virtual(&second, "close", "()V", ()).await?;

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }

    #[test]
    fn stream_constructors_validate_mode_and_resource_reset_restores_offset() {
        let result = run_jvm_test(
            Box::new([Box::new([XFile::as_proto(), FileInputStream::as_proto(), FileOutputStream::as_proto()])]),
            |jvm| async move {
                let write: i32 = jvm.get_static_field("com/xce/io/XFile", "WRITE", "I").await?;
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "write-only.dat").await?.into();
                let write_only: ClassInstanceRef<XFile> = jvm.new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, write)).await?.into();
                let available_result: JvmResult<i32> = jvm.invoke_virtual(&write_only, "available", "()I", ()).await;
                let Err(JavaError::JavaException(exception)) = available_result else {
                    panic!("write-only XFile reported readable bytes");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let input_result: JvmResult<ClassInstanceRef<FileInputStream>> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Lcom/xce/io/XFile;)V", (write_only.clone(),))
                    .await
                    .map(Into::into);
                let Err(JavaError::JavaException(exception)) = input_result else {
                    panic!("FileInputStream accepted a write-only XFile");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let mut one_byte = jvm.instantiate_array("B", 1).await?;
                jvm.store_array(&mut one_byte, 0, [1_i8]).await?;
                let _: i32 = jvm.invoke_virtual(&write_only, "write", "([BII)I", (one_byte, 0, 1)).await?;
                let _: () = jvm.invoke_virtual(&write_only, "close", "()V", ()).await?;

                let read: i32 = jvm.get_static_field("com/xce/io/XFile", "READ", "I").await?;
                let name: ClassInstanceRef<String> = JavaLangString::from_rust_string(&jvm, "write-only.dat").await?.into();
                let read_only: ClassInstanceRef<XFile> = jvm.new_class("com/xce/io/XFile", "(Ljava/lang/String;I)V", (name, read)).await?.into();
                let output_result: JvmResult<ClassInstanceRef<FileOutputStream>> = jvm
                    .new_class("com/xce/io/FileOutputStream", "(Lcom/xce/io/XFile;)V", (read_only,))
                    .await
                    .map(Into::into);
                let Err(JavaError::JavaException(exception)) = output_result else {
                    panic!("FileOutputStream accepted a read-only XFile");
                };
                assert!(jvm.is_instance(&*exception, "java/io/IOException"));

                let mut resource: ClassInstanceRef<XFile> = jvm.new_class("com/xce/io/XFile", "(I)V", (0,)).await?.into();
                let mut bytes = jvm.instantiate_array("B", 2).await?;
                jvm.store_array(&mut bytes, 0, [7_i8, 8]).await?;
                let stream = jvm.new_class("java/io/ByteArrayInputStream", "([B)V", (bytes,)).await?;
                jvm.put_field(&mut resource, "type", "I", FILE_JAR).await?;
                jvm.put_field(&mut resource, "mode", "I", READ_RESOURCE).await?;
                jvm.put_field(&mut resource, "is", "Ljava/io/InputStream;", stream).await?;

                let input: ClassInstanceRef<FileInputStream> = jvm
                    .new_class("com/xce/io/FileInputStream", "(Lcom/xce/io/XFile;)V", (resource.clone(),))
                    .await?
                    .into();
                let _: () = jvm.invoke_virtual(&input, "mark", "(I)V", (2,)).await?;
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, 7);
                assert_eq!(jvm.get_field::<i32>(&resource, "offset", "I").await?, 1);
                let _: () = jvm.invoke_virtual(&input, "reset", "()V", ()).await?;
                assert_eq!(jvm.get_field::<i32>(&resource, "offset", "I").await?, 0);
                assert_eq!(jvm.invoke_virtual::<_, i32>(&input, "read", "()I", ()).await?, 7);

                Ok(())
            },
        );

        assert!(result.is_ok(), "JVM test failed: {result:?}");
    }
}
