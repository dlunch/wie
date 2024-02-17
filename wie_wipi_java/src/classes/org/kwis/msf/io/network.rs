use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, JvmResult};

use crate::context::{WIPIJavaClassProto, WIPIJavaContext};

// class org.kwis.msf.io.Network
pub struct Network {}

impl Network {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("disconnect", "()V", Self::disconnect, MethodAccessFlags::NATIVE)],
            fields: vec![],
        }
    }

    async fn disconnect(_: &Jvm, _: &mut WIPIJavaContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
