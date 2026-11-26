use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class com.skt.m.Call
#[java_class(name = "com/skt/m/Call", public, final)]
pub struct Call;

#[java_methods]
impl Call {
    #[method(public, static)]
    async fn connect(_jvm: &Jvm, _context: &mut WieJvmContext, phone_number: ClassInstanceRef<String>) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::connect({phone_number:?})");

        Ok(false)
    }

    #[method(public, static)]
    async fn disconnect(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::disconnect()");

        Ok(false)
    }

    #[method(public, static)]
    async fn is_supported(_jvm: &Jvm, _context: &mut WieJvmContext) -> JvmResult<bool> {
        tracing::warn!("stub com.skt.m.Call::isSupported()");

        Ok(false)
    }
}
