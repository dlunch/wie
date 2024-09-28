use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.xce.io.XFile
pub struct XFile {}

impl XFile {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/io/XFile",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "exists",
                "(Ljava/lang/String;)Z",
                Self::exists,
                MethodAccessFlags::STATIC,
            )],
            fields: vec![],
        }
    }

    async fn exists(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub com.xce.io.XFile::exists({:?})", name);

        Ok(false)
    }
}
