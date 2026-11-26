use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class net.wie.WieError
#[java_class(name = "net/wie/WieError", extends = "java/lang/Error", public)]
pub struct WieError;

#[java_methods]
impl WieError {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("net.wie.WieError::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Error", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("net.wie.WieError::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/Error", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
