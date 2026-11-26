use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// interface javax.microedition.media.control.VolumeControl
#[java_class(
    name = "javax/microedition/media/control/VolumeControl",
    implements("javax/microedition/media/Control"),
    public,
    interface,
    abstract
)]
pub struct VolumeControl;

#[java_methods]
impl VolumeControl {
    #[method(public)]
    async fn set_mute(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, mute: bool) -> Result<()>;

    #[method(public)]
    async fn is_muted(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<bool>;

    #[method(public)]
    async fn set_level(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, level: i32) -> Result<i32>;

    #[method(public)]
    async fn get_level(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<i32>;
}
