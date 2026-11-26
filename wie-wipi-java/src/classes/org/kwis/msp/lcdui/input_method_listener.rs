use jvm::{Array, ClassInstanceRef, JavaChar, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface org.kwis.msp.lcdui.InputMethodListener
#[java_class(name = "org/kwis/msp/lcdui/InputMethodListener", public, interface, abstract)]
pub struct InputMethodListener;

#[java_methods]
impl InputMethodListener {
    #[method(public)]
    async fn notify_text_changed(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        text: ClassInstanceRef<Array<JavaChar>>,
        length: i32,
        mode: i32,
    ) -> JvmResult<()>;
}
