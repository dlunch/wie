use alloc::vec;

use java_class_proto::JavaMethodProto;
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use crate::context::{SKVMJavaClassProto, SKVMJavaContext};

// class com.skt.m.Device
pub struct Device {}

impl Device {
    pub fn as_proto() -> SKVMJavaClassProto {
        SKVMJavaClassProto {
            name: "com/skt/m/Device",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<init>", "()V", Self::init, Default::default())],
            fields: vec![],
        }
    }

    async fn init(_jvm: &Jvm, _context: &mut SKVMJavaContext, this: ClassInstanceRef<Self>) -> JvmResult<()> {
        tracing::debug!("com.skt.m.Device::<init>({:?})", &this);

        Ok(())
    }
}
