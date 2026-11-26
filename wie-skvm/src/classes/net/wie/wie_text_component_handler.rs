use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class net.wie.WieTextComponentHandler
#[java_class(name = "net/wie/WieTextComponentHandler", extends = "com/xce/lcdui/TextComponentHandler", public)]
pub struct WieTextComponentHandler;

#[java_methods]
impl WieTextComponentHandler {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("net.wie.WieTextComponentHandler::<init>({this:?})");
        jvm.invoke_special(&this, "com/xce/lcdui/TextComponentHandler", "<init>", "()V", ()).await
    }
}
