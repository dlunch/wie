use alloc::{format, vec};

use java_runtime::classes::java::lang::String;
use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult, JvmClassInstanceHandle};
use jvm::Jvm;

use crate::{WieClassProto, WieContext};

// class org.kwis.msp.io.FileSystem
pub struct FileSystem {}

impl FileSystem {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
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

    async fn is_file(_jvm: &mut Jvm, _: &mut WieContext, name: JvmClassInstanceHandle<String>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::is_file({:?})", &name);

        Ok(false)
    }

    async fn is_directory(_jvm: &mut Jvm, _: &mut WieContext, name: JvmClassInstanceHandle<String>, flag: i32) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::isDirectory({:?}, {:?})", &name, flag);

        Ok(true)
    }

    async fn exists(jvm: &mut Jvm, context: &mut WieContext, name: JvmClassInstanceHandle<String>) -> JavaResult<bool> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::exists({:?})", &name);

        let filename = String::to_rust_string(jvm, &name)?;

        // emulating filesystem by resource..
        let filename_on_resource = format!("P{}", filename);

        let id = context.system().resource().id(&filename_on_resource);

        Ok(id.is_some())
    }

    async fn available(_: &mut Jvm, _: &mut WieContext) -> JavaResult<i32> {
        tracing::warn!("stub org.kwis.msp.io.FileSystem::available()");

        Ok(0x1000000) // TODO temp
    }
}
