use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use crate::context::{MIDPJavaClassProto, MIDPJavaContext};

// class javax.microedition.midlet.MIDlet
pub struct MIDlet {}

impl MIDlet {
    pub fn as_proto() -> MIDPJavaClassProto {
        MIDPJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new_abstract("startApp", "([Ljava/lang/String;)V", Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(_jvm: &Jvm, _context: &mut MIDPJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("javax.microedition.midlet.MIDlet::<init>({:?})", &this);

        Ok(())
    }
}
