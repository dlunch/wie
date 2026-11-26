use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::Object;

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.lwc.GrabKeyListener
#[java_class(name = "org/kwis/msp/lwc/GrabKeyListener", public, interface, abstract)]
pub struct GrabKeyListener;

#[java_methods]
impl GrabKeyListener {
    #[method(public)]
    async fn grab_key_notify(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        r#type: i32,
        chr: i32,
        object: ClassInstanceRef<Object>,
    ) -> JvmResult<bool>;
}
