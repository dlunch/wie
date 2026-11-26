use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use super::SMSMessage;

// interface com.skt.m.SMSListener
#[java_class(name = "com/skt/m/SMSListener", public, interface, abstract)]
pub struct SMSListener;

#[java_methods]
impl SMSListener {
    #[method(public)]
    async fn on_message(
        jvm: &Jvm,
        _context: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        message: ClassInstanceRef<SMSMessage>,
    ) -> JvmResult<()>;
}
