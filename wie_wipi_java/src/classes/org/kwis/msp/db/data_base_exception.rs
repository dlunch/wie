use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.db.DataBaseException
pub struct DataBaseException;

impl DataBaseException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBaseException",
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseException::<init>({:?})", &this);

        let _: () = jvm.invoke_special(&this, "java/lang/Exception", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseException::<init>({:?}, {:?})", &this, &message);

        let _: () = jvm
            .invoke_special(&this, "java/lang/Exception", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
