use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_runtime::classes::java::{io::FileDescriptor, lang::String};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::com::xce::io::x_file::XFile;

// class com.xce.io.FileOutputStream
pub struct FileOutputStream;

impl FileOutputStream {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/FileOutputStream",
            parent_class: Some("java/io/OutputStream"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Lcom/xce/io/XFile;)V", Self::init_with_file, Default::default()),
                JavaMethodProto::new("write", "(I)V", Self::write, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![JavaFieldProto::new("os", "Ljava/io/OutputStream;", Default::default())],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, mut this: ClassInstanceRef<Self>, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::<init>({:?}, {:?})", this, name);

        let _: () = jvm.invoke_special(&this, "java/io/OutputStream", "<init>", "()V", ()).await?;

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let os = jvm.new_class("java/io/FileOutputStream", "(Ljava/io/File;)V", (file,)).await?;

        jvm.put_field(&mut this, "os", "Ljava/io/OutputStream;", os).await?;

        Ok(())
    }

    async fn init_with_file(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        mut this: ClassInstanceRef<Self>,
        file: ClassInstanceRef<XFile>,
    ) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::<init>({:?})", file);

        let raf = XFile::raf(jvm, file).await?;
        let fd: ClassInstanceRef<FileDescriptor> = jvm.invoke_virtual(&raf, "getFD", "()Ljava/io/FileDescriptor;", ()).await?;
        let os = jvm.new_class("java/io/FileOutputStream", "(Ljava/io/FileDescriptor;)V", (fd,)).await?;

        jvm.put_field(&mut this, "os", "Ljava/io/OutputStream;", os).await?;

        Ok(())
    }

    async fn write(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>, byte: i32) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::write({:?}, {:?})", this, byte);

        let os = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;
        let _: () = jvm.invoke_virtual(&os, "write", "(I)V", (byte,)).await?;

        Ok(())
    }

    async fn close(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.xce.io.FileOutputStream::close({:?})", this);

        let os = jvm.get_field(&this, "os", "Ljava/io/OutputStream;").await?;
        let _: () = jvm.invoke_virtual(&os, "close", "()V", ()).await?;

        Ok(())
    }
}
