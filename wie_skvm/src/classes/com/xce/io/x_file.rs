use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::{
    io::{InputStream, OutputStream},
    lang::String,
};
use jvm::{Array, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

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
            ],
            fields: vec![
                JavaFieldProto::new("is", "Ljava/io/InputStream;", Default::default()),
                JavaFieldProto::new("os", "Ljava/io/OutputStream;", Default::default()),
            ],
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

        if mode == 8 {
            // READ_RESOURCE
            let class = jvm.invoke_virtual(&this, "getClass", "()Ljava/lang/Class;", ()).await?;
            let resource_stream: ClassInstanceRef<InputStream> = jvm
                .invoke_virtual(&class, "getResourceAsStream", "(Ljava/lang/String;)Ljava/io/InputStream;", (name,))
                .await?;

            jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", resource_stream).await?;
        } else {
            let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;

            let is = jvm.new_class("java/io/FileInputStream", "(Ljava/io/File;)V", (file.clone(),)).await?;
            let os = jvm.new_class("java/io/FileOutputStream", "(Ljava/io/File;)V", (file,)).await?;

            jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", is).await?;
            jvm.put_field(&mut this, "os", "Ljava/io/OutputStream;", os).await?;
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

        let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
        let available = jvm.invoke_virtual(&is, "available", "()I", ()).await?;

        Ok(available)
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

        let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
        let read: i32 = jvm.invoke_virtual(&is, "read", "([BII)I", (data, offset, length)).await?;

        Ok(read)
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

        let os: ClassInstanceRef<OutputStream> = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;

        let _: () = jvm.invoke_virtual(&os, "write", "([BII)V", (data, offset, length)).await?;

        Ok(length)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::close({:?})", this);

        let is: ClassInstanceRef<InputStream> = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
        let os: ClassInstanceRef<OutputStream> = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;

        if !is.is_null() {
            let _: () = jvm.invoke_virtual(&is, "close", "()V", ()).await?;
        }
        if !os.is_null() {
            let _: () = jvm.invoke_virtual(&os, "close", "()V", ()).await?;
        }

        Ok(())
    }

    pub async fn input_stream(jvm: &Jvm, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<InputStream>> {
        jvm.get_field(&this, "is", "Ljava/io/InputStream;").await
    }

    pub async fn output_stream(jvm: &Jvm, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<OutputStream>> {
        jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await
    }
}
