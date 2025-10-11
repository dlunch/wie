use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.SchemeNotFoundException
pub struct SchemeNotFoundException;

impl SchemeNotFoundException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/SchemeNotFoundException",
            parent_class: Some("java/io/IOException"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, Default::default()),
            ],
            fields: vec![],
            access_flags: Default::default(),
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<SchemeNotFoundException>) -> Result<()> {
        tracing::debug!("org.kwis.msf.io.SchemeNotFoundException::<init>({this:?})");

        let _: () = jvm.invoke_special(&this, "java/io/IOException", "<init>", "()V", ()).await?;

        Ok(())
    }

    async fn init_with_message(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        this: ClassInstanceRef<SchemeNotFoundException>,
        message: ClassInstanceRef<String>,
    ) -> Result<()> {
        tracing::debug!("org.kwis.msf.io.SchemeNotFoundException::<init>({this:?}, {message:?})");

        let _: () = jvm
            .invoke_special(&this, "java/io/IOException", "<init>", "(Ljava/lang/String;)V", (message,))
            .await?;

        Ok(())
    }
}
