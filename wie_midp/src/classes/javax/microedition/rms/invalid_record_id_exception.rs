use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::lang::String;

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.rms.InvalidRecordIDException
pub struct InvalidRecordIDException;

impl InvalidRecordIDException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/rms/InvalidRecordIDException",
            parent_class: Some("javax/microedition/rms/RecordStoreException"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, MethodAccessFlags::PUBLIC),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, MethodAccessFlags::PUBLIC),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("javax.microedition.rms.InvalidRecordIDException::<init>({this:?})");

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/rms/RecordStoreException", "<init>", "()V", ())
            .await?;

        Ok(())
    }

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
