use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.lcdui.JletEventListener
#[java_class(name = "org/kwis/msp/lcdui/JletEventListener", public, interface, abstract)]
pub struct JletEventListener;

#[java_methods]
impl JletEventListener {
    #[method(public)]
    async fn notify_event(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, r#type: i32, param1: i32, param2: i32) -> JvmResult<()>;
}
