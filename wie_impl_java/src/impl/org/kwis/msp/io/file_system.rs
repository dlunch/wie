use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaMethodProto},
    proxy::JvmClassInstanceProxy,
    r#impl::java::lang::String,
    JavaContext, JavaMethodFlag, JavaResult,
};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem {}

impl FileSystem {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("isFile", "(Ljava/lang/String;)Z", Self::is_file, JavaMethodFlag::STATIC),
                JavaMethodProto::new("isDirectory", "(Ljava/lang/String;I)Z", Self::is_directory, JavaMethodFlag::STATIC),
                JavaMethodProto::new("exists", "(Ljava/lang/String;)Z", Self::exists, JavaMethodFlag::STATIC),
                JavaMethodProto::new("available", "()I", Self::available, JavaMethodFlag::STATIC),
            ],
            fields: vec![],
        }
    }

    async fn is_file(_context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::is_file({:?})", &name);

        Ok(0)
    }

    async fn is_directory(_context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>, flag: i32) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isDirectory({:?}, {:?})", &name, flag);

        Ok(1)
    }

    async fn exists(_context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::exists({:?})", &name);

        Ok(0)
    }

    async fn available(_context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available({:?})", &name);

        Ok(0x1000000) // TODO temp
    }
}
