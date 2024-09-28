use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::MethodAccessFlags;
use jvm::{Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class org.kwis.msf.io.Network
pub struct Network;

impl Network {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Network",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("disconnect", "()V", Self::disconnect, MethodAccessFlags::NATIVE)],
            fields: vec![],
        }
    }

    async fn disconnect(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
