use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use java_runtime::classes::java::lang::String;
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
                JavaMethodProto::new("write", "([BII)I", Self::write, Default::default()),
                JavaMethodProto::new("close", "()V", Self::close, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        name: ClassInstanceRef<String>,
        mode: i32,
    ) -> JvmResult<()> {
        tracing::warn!("stub com.xce.io.XFile::<init>({:?}, {:?}, {:?})", this, name, mode);

        Ok(())
    }

    async fn exists(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub com.xce.io.XFile::exists({:?})", name);

        Ok(false)
    }

    async fn filesize(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::filesize({:?})", name);

        Ok(0)
    }

    async fn unlink(_jvm: &Jvm, _context: &mut WieJvmContext, name: ClassInstanceRef<String>) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::unlink({:?})", name);

        Ok(0)
    }

    async fn write(
        _jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        data: ClassInstanceRef<Array<i8>>,
        offset: i32,
        length: i32,
    ) -> JvmResult<i32> {
        tracing::warn!("stub com.xce.io.XFile::write({:?}, {:?}, {}, {})", this, data, offset, length);

        Ok(0)
    }

    async fn close(_jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::warn!("stub com.xce.io.XFile::close({:?})", this);

        Ok(())
    }
}
