use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class com.skt.m.Call
pub struct Call;

impl Call {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/Call",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "connect",
                    "(Ljava/lang/String;)Z",
                    Self::connect,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "disconnect",
                    "()Z",
                    Self::disconnect,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "isSupported",
                    "()Z",
                    Self::is_supported,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::FINAL,
        }
    }

    async fn connect(_jvm: &Jvm, _context: &mut WieJvmContext, phone_number: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::connect({phone_number:?})");

        Ok(false)
    }

    async fn disconnect(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::disconnect()");

        Ok(false)
    }

    async fn is_supported(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::isSupported()");

        Ok(false)
    }
}
