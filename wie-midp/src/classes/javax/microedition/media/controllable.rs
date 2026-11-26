use jvm::{Array, ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::media::Control;

// interface javax.microedition.media.Controllable
#[java_class(name = "javax/microedition/media/Controllable", public, interface, abstract)]
pub struct Controllable;

#[java_methods]
impl Controllable {
    #[method(public)]
    async fn get_control(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<Self>,
        control_type: ClassInstanceRef<String>,
    ) -> Result<ClassInstanceRef<Control>>;

    #[method(public)]
    async fn get_controls(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<ClassInstanceRef<Array<Control>>>;
}
