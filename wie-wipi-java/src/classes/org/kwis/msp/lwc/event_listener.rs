use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.lwc.EventListener
#[java_class(name = "org/kwis/msp/lwc/EventListener", public, interface, abstract)]
pub struct EventListener;

#[java_methods]
impl EventListener {
    #[method(public)]
    async fn event_notify(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        event_type: i32,
        subtype: i32,
        param1: i32,
        param2: i32,
        object: ClassInstanceRef<Object>,
    ) -> JvmResult<bool>;
}
