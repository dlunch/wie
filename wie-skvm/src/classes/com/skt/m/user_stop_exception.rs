use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class com.skt.m.UserStopException
#[java_class(name = "com/skt/m/UserStopException", extends = "java/lang/Exception", public)]
pub struct UserStopException;

#[java_methods]
impl UserStopException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _context: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.UserStopException::<init>({this:?})");

        jvm.invoke_special(&this, "java/lang/Exception", "<init>", "()V", ()).await
    }
}
