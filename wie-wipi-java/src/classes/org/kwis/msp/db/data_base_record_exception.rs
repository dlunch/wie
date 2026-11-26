use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::WieJvmContext;

// class org.kwis.msp.db.DataBaseRecordException
#[java_class(name = "org/kwis/msp/db/DataBaseRecordException", extends = "java/lang/Exception", public)]
pub struct DataBaseRecordException;

#[java_methods]
impl DataBaseRecordException {
    #[method(constructor, public)]
    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBaseRecordException>) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseRecordException::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/lang/Exception", "<init>", "()V", ()).await?;

        Ok(())
    }

    #[method(constructor, public)]
    async fn init_with_message(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<DataBaseRecordException>,
        message: ClassInstanceRef<String>,
    ) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseRecordException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/lang/Exception", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
