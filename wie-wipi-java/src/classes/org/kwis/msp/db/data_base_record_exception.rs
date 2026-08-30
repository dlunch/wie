use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msp.db.DataBaseRecordException
pub struct DataBaseRecordException;

impl DataBaseRecordException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBaseRecordException",
            parent_class: Some("org/kwis/msp/db/DataBaseException"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<DataBaseRecordException>) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseRecordException::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/db/DataBaseException", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn init_with_message(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<DataBaseRecordException>,
        message: ClassInstanceRef<String>,
    ) -> Result<()> {
        tracing::debug!("org.kwis.msp.db.DataBaseRecordException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "org/kwis/msp/db/DataBaseException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
