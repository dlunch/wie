use alloc::vec;

use crate::base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult};

// class org.kwis.msf.io.Network
pub struct Network {}

impl Network {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("disconnect", "()V", Self::disconnect, JavaMethodFlag::NATIVE)],
            fields: vec![],
        }
    }

    async fn disconnect(_: &mut dyn JavaContext) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
