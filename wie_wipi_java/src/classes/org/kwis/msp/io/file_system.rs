use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{runtime::JavaLangString, ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem {}

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
                JavaMethodProto::new("available", "()I", Self::available, MethodAccessFlags::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn is_file(_jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::is_file({:?})", &name);

        Ok(false)
    }

    async fn is_directory(_jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, flag: i32) -> JvmResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isDirectory({:?}, {:?})", &name, flag);

        Ok(true)
    }

    async fn exists(jvm: &Jvm, context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::debug!("org.kwis.msp.io.FileSystem::exists({:?})", &name);

        let filename = JavaLangString::to_rust_string(jvm, &name).await?;

        let exists = context.system().filesystem().read(&filename).is_some();

        Ok(exists)
    }

    async fn available(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }
}
