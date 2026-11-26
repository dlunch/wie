use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class com.skt.m.ResourceAllocException
#[java_class(name = "com/skt/m/ResourceAllocException", extends = "java/lang/Exception", public)]
pub struct ResourceAllocException;

#[java_methods]
impl ResourceAllocException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.ResourceAllocException::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Exception", "<init>", "()V", ()).await
    }
}
