use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::{
    io::{DataInputStream, DataOutputStream, InputStream, OutputStream},
    lang::String,
};

use wie_jvm_support::WieJvmContext;

use crate::classes::javax::microedition::io::Connection;

// class javax.microedition.io.Connector
#[java_class(name = "javax/microedition/io/Connector", public)]
pub struct Connector;

#[java_methods]
impl Connector {
    // No protocol is supported yet, so every open fails the way the specification requires for an unsupported protocol.
    #[method(public, static)]
    async fn open(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(name = "open", public, static)]
    async fn open_with_mode(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, mode: i32) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?}, {mode})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(name = "open", public, static)]
    async fn open_with_mode_and_timeouts(
        jvm: &Jvm,
        _: &mut WieJvmContext,
        name: ClassInstanceRef<String>,
        mode: i32,
        timeouts: bool,
    ) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?}, {mode}, {timeouts})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(public, static)]
    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<InputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(public, static)]
    async fn open_data_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataInputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(public, static)]
    async fn open_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<OutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    #[method(public, static)]
    async fn open_data_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataOutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }
}
