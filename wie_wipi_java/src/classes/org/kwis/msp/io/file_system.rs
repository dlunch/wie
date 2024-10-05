use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem;

impl FileSystem {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/io/FileSystem",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("isFile", "(Ljava/lang/String;)Z", Self::is_file, MethodAccessFlags::STATIC),
                JavaMethodProto::new("isDirectory", "(Ljava/lang/String;I)Z", Self::is_directory, MethodAccessFlags::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, MethodAccessFlags::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;I)Z", Self::exists_with_flag, MethodAccessFlags::STATIC),
                JavaMethodProto::new("mkdir", "(Ljava/lang/String;I)V", Self::mkdir, MethodAccessFlags::STATIC),
                JavaMethodProto::new("available", "()I", Self::available, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn is_file(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::is_file({:?})", &name);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_file = jvm.invoke_virtual(&file, "isFile", "()Z", ()).await?;

        Ok(is_file)
    }

    async fn is_directory(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::isDirectory({:?}, {:?})", &name, flag);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let is_directory = jvm.invoke_virtual(&file, "isDirectory", "()Z", ()).await?;

        Ok(is_directory)
    }

    async fn exists(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({:?})", &name);

        jvm.invoke_static("org/kwis/msp/io/FileSystem", "exists", "(Ljava/lang/String;I)Z", (name, 0))
            .await
    }

    async fn exists_with_flag(jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({:?}, {:?})", &name, flag);

        let file = jvm.new_class("java/io/File", "(Ljava/lang/String;)V", (name,)).await?;
        let exists = jvm.invoke_virtual(&file, "exists", "()Z", ()).await?;

        Ok(exists)
    }

    async fn mkdir(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::mkdir({:?}, {:?})", &name, flag);

        Ok(())
    }

    async fn available(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }
}
