use alloc::vec;

use java_runtime_base::{JavaMethodFlag, JavaMethodProto, JavaResult};
use jvm::Jvm;

use crate::{WieClassProto, WieContext};

// class org.kwis.msf.io.Network
pub struct Network {}

impl Network {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("disconnect", "()V", Self::disconnect, JavaMethodFlag::NATIVE)],
            fields: vec![],
        }
    }

    async fn disconnect(_: &mut Jvm, _: &mut WieContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
