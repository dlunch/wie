use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::{
    io::{InputStream, RandomAccessFile},
    lang::String,
};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult, runtime::JavaLangString};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

const READ_RESOURCE: i32 = 8;

// class com.xce.io.XFile
pub struct XFile;

impl XFile {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/XFile",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;I)V", Self::init, Default::default()),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, MethodAccessFlags::STATIC),
                JavaMethodProto::new("filesize", "(Ljava/lang/String;)I", Self::filesize, MethodAccessFlags::STATIC),
                JavaMethodProto::new("unlink", "(Ljava/lang/String;)I", Self::unlink, MethodAccessFlags::STATIC),
                JavaMethodProto::new("available", "()I", Self::available, Default::default()),
                JavaMethodProto::new("read", "([BII)I", Self::read, Default::default()),
                JavaMethodProto::new("write", "([BII)I", Self::write, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
                JavaMethodProto::new("seek", "(II)I", Self::seek, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("mode", "I", Default::default()),
                JavaFieldProto::new("is", "Ljava/io/InputStream;", Default::default()),
                JavaFieldProto::new("raf", "Ljava/io/RandomAccessFile;", Default::default()),
            ],
            access_flags: Default::default(),
        }
    }

    async fn init(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::<init>({:?}, {:?}, {:?})", this, name, mode);

        let _: () = jvm.invoke_special(&this, "java/lang/Object", "<init>", "()V", ()).await?;

        if mode == READ_RESOURCE {
            let class = jvm.invoke_virtual(&this, "getClass", "()Ljava/lang/Class;", ()).await?;
            let resource_stream: ClassInstanceRef<InputStream> = jvm
                .invoke_virtual(&class, "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;", (name,))
                .await?;

            jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", resource_stream).await?;
            jvm.put_field(&mut this, "mode", "I", mode).await?;
        } else {
            let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;

            let file_mode = JavaLangString::from_rust_string(jvm, "rw").await?;

            let raf = jvm
                .new_class("java/io/RandomAccessFile", "(Ljava/io/File;Ljava/lang/String;)V", (file, file_mode))
                .await?;
            jvm.put_field(&mut this, "raf", "Ljava/io/RandomAccessFile;", raf).await?;
            jvm.put_field(&mut this, "mode", "I", mode).await?;
        }

        Ok(())
    }

    async fn exists(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("com.xce.io.XFile::exists({:?})", name);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;

        Ok(exists)
    }

    async fn filesize(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::filesize({:?})", name);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let size: i64 = jvm.invoke_virtual(&file, "length", "()J", ()).await?;

        Ok(size as _)
    }

    async fn unlink(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::unlink({:?})", name);

        Ok(0)
    }

    async fn available(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::available({this:?})");

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            let available = jvm.invoke_virtual(&is, "available", "()I", ()).await?;

            Ok(available)
        } else {
            let raf: ClassInstanceRef<RandomAccessFile> = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let file_length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;
            let file_pointer: i64 = jvm.invoke_virtual(&raf, "getFilePointer", "()J", ()).await?;

            Ok((file_length - file_pointer) as i32)
        }
    }

    async fn read(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::read({:?}, {:?}, {}, {})", this, data, offset, length);

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;

        if mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            let read: i32 = jvm.invoke_virtual(&is, "read", "([BII)I", (data, offset, length)).await?;

            Ok(read)
        } else {
            let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let read: i32 = jvm.invoke_virtual(&raf, "read", "([BII)I", (data, offset, length)).await?;

            Ok(read)
        }
    }

    async fn write(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::write({:?}, {:?}, {}, {})", this, data, offset, length);

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode == READ_RESOURCE {
            return Err(jvm.exception("java/io/IOException", "File not opened for writing").await);
        }

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (data, offset, length)).await?;

        Ok(length)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::close({:?})", this);

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode == READ_RESOURCE {
            let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
            let _: () = jvm.invoke_virtual(&is, "close", "()V", ()).await?;
        } else {
            let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
            let _: () = jvm.invoke_virtual(&raf, "close", "()V", ()).await?;
        }

        Ok(())
    }

    async fn seek(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, n: i32, whence: i32) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.XFile::seek({this:?}, {n}, {whence})");

        let mode: i32 = jvm.get_field(&this, "mode", "I").await?;
        if mode == READ_RESOURCE {
            return Err(jvm.exception("java/io/IOException", "File not opened for writing").await);
        }

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let new_pos = match whence {
            0 => n as i64,
            1 => {
                let current: i64 = jvm.invoke_virtual(&raf, "getFilePointer", "()J", ()).await?;
                current + n as i64
            }
            2 => {
                let length: i64 = jvm.invoke_virtual(&raf, "length", "()J", ()).await?;
                length + n as i64
            }
            _ => return Err(jvm.exception("java/io/IOException", "Invalid whence").await),
        };

        let _: () = jvm.invoke_virtual(&raf, "seek", "(J)V", (new_pos,)).await?;

        Ok(new_pos as i32)
    }

    pub async fn raf(jvm: &Jvm, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<RandomAccessFile>> {
        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;

        Ok(raf)
    }
}
