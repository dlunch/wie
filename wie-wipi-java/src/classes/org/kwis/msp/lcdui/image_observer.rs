use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msp::lcdui::Image;

// interface org.kwis.msp.lcdui.ImageObserver
#[java_class(name = "org/kwis/msp/lcdui/ImageObserver", public, interface, abstract)]
pub struct ImageObserver;

#[java_methods]
impl ImageObserver {
    #[method(public)]
    async fn notify(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, image: ClassInstanceRef<Image>, status: i32) -> JvmResult<()>;
}
