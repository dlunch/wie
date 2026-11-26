use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class org.kwis.msf.io.SchemeNotFoundException
#[java_class(name = "org/kwis/msf/io/SchemeNotFoundException", extends = "java/io/IOException", public)]
pub struct SchemeNotFoundException;

#[java_methods]
impl SchemeNotFoundException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("org.kwis.msf.io.SchemeNotFoundException::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/io/IOException", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("org.kwis.msf.io.SchemeNotFoundException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/io/IOException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
