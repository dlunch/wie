use jvm::{Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};

use wie_jvm_support::WieJvmContext;

// class org.kwis.msf.io.Network
#[java_class(name = "org/kwis/msf/io/Network", public)]
pub struct Network;

#[java_methods]
impl Network {
    #[method(public, native, static)]
    async fn connect(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<i32> {
        tracing::warn!("stub org.kwis.msf.io.Network::connect()");

        Ok(-1)
    }

    #[method(public, native, static)]
    async fn disconnect(_: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::warn!("stub org.kwis.msf.io.Network::disconnect()");

        Ok(())
    }
}
