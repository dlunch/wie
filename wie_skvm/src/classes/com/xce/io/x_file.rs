use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::{io::File, lang::String};
use jvm::{runtime::JavaLangString, Array, ClassInstanceRef, Jvm, Result as JvmResult};

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
                JavaMethodProto::new("write", "([BII)I", Self::write, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![
                JavaFieldProto::new("file", "Ljava/io/File;", Default::default()),
                JavaFieldProto::new("raf", "Ljava/io/RandomAccessFile;", Default::default()),
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

        let mode_string = JavaLangString::from_rust_string(
            jvm,
            match mode {
                1 => "r",
                2 => "rw",
                3 => "rw",
                _ => return Err(jvm.exception("java/io/IOException", "Invalid mode").await),
            },
        )
        .await?;

        let raf = jvm
            .new_class(
                "java/io/RandomAccessFile",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                (name.clone(), mode_string),
            )
            .await?;
        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;

        jvm.put_field(&mut this, "file", "Ljava/io/File;", file).await?;
        jvm.put_field(&mut this, "raf", "Ljava/io/RandomAccessFile;", raf).await?;

        Ok(())
    }

    async fn exists(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("com.xce.io.XFile::exists({:?})", name);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;

        Ok(exists)
    }

    async fn filesize(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::filesize({:?})", name);

        Err(jvm.exception("java/io/IOException", "File not found").await)
    }

    async fn unlink(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::unlink({:?})", name);

        Ok(0)
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

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "write", "([BII)V", (data, offset, length)).await?;

        Ok(length)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.XFile::close({:?})", this);

        let raf = jvm.get_field(&this, "raf", "Ljava/io/RandomAccessFile;").await?;
        let _: () = jvm.invoke_virtual(&raf, "close", "()V", ()).await?;

        Ok(())
    }

    pub async fn file(jvm: &Jvm, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<File>> {
        jvm.get_field(&this, "file", "Ljava/io/File;").await
    }
}
