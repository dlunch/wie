use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::xce::io::x_file::XFile;

// class com.xce.io.FileInputStream
pub struct FileInputStream;

impl FileInputStream {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/FileInputStream",
            parent_class: Some("java/io/InputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Lcom/xce/io/XFile;)V", Self::init_with_file, Default::default()),
                JavaMethodProto::new("available", "()I", Self::available, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("is", "Ljava/io/InputStream;", Default::default())],
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({:?}, {:?})", this, name);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is = jvm.new_class("java/io/FileInputStream", "(Ljava/io/File;)V", (file,)).await?;

        jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", is).await?;

        Ok(())
    }

    async fn init_with_file(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        file: ClassInstanceRef<XFile>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::<init>({:?}, {:?})", this, file);

        let file = XFile::file(jvm, file).await?;
        let is = jvm.new_class("java/io/FileInputStream", "(Ljava/io/File;)V", (file,)).await?;
        jvm.put_field(&mut this, "is", "Ljava/io/InputStream;", is).await?;

        Ok(())
    }

    async fn available(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::debug!("com.xce.io.FileInputStream::available({:?})", this);

        let is = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
        let available = jvm.invoke_virtual(&is, "available", "()I", ()).await?;

        Ok(available)
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileInputStream::close({:?})", this);

        let is = jvm.get_field(&this, "is", "Ljava/io/InputStream;").await?;
        let _: () = jvm.invoke_virtual(&is, "close", "()V", ()).await?;

        Ok(())
    }
}
