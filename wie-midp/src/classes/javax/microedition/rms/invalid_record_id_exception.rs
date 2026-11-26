use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class javax.microedition.rms.InvalidRecordIDException
#[java_class(
    name = "javax/microedition/rms/InvalidRecordIDException",
    extends = "javax/microedition/rms/RecordStoreException",
    public
)]
pub struct InvalidRecordIDException;

#[java_methods]
impl InvalidRecordIDException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("javax.microedition.rms.InvalidRecordIDException::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/rms/RecordStoreException", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("javax.microedition.rms.InvalidRecordIDException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/rms/RecordStoreException",
                "<init>",
                "(Ljava/lang/String;)V",
                (message,),
            )
            .await?;

        Ok(())
    }
}
