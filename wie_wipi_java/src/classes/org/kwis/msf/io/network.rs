use alloc::vec;

use java_class_proto::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{WIPIJavaClassProto, WIPIJavaContxt};

// class org.kwis.msf.io.Network
pub struct Network {}

impl Network {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("disconnect", "()V", Self::disconnect, JavaMethodFlag::NATIVE)],
            fields: vec![],
        }
    }

    async fn disconnect(_: &mut Jvm, _: &mut WIPIJavaContxt) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
