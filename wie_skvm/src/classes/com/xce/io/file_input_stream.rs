use alloc::vec;

use java_class_proto::JavaMethodProto;
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
            fields: vec![],
        }
    }

    async fn init(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.io.FileInputStream::<init>({:?})", name);

        Ok(())
    }

    async fn init_with_file(_jvm: &Jvm, _context: &mut WieJvmContext, file: ClassInstanceRef<XFile>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.io.FileInputStream::<init>({:?})", file);

        Ok(())
    }

    async fn available(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.FileInputStream::available({:?})", this);

        Ok(0)
    }

    async fn close(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.io.FileInputStream::close({:?})", this);

        Ok(())
    }
}
