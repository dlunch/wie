use alloc::vec;

use jvm::{ClassInstanceRef, Jvm, Result};
use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};
use rustjava_runtime::classes::java::{
    io::{DataInputStream, DataOutputStream, InputStream, OutputStream},
    lang::String,
};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

use crate::classes::javax::microedition::io::Connection;

// class javax.microedition.io.Connector
pub struct Connector;

impl Connector {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/io/Connector",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;)Ljavax/microedition/io/Connection;",
                    Self::open,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;I)Ljavax/microedition/io/Connection;",
                    Self::open_with_mode,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "open",
                    "(Ljava/lang/String;IZ)Ljavax/microedition/io/Connection;",
                    Self::open_with_mode_and_timeouts,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openInputStream",
                    "(Ljava/lang/String;)Ljava/io/InputStream;",
                    Self::open_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataInputStream",
                    "(Ljava/lang/String;)Ljava/io/DataInputStream;",
                    Self::open_data_input_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openOutputStream",
                    "(Ljava/lang/String;)Ljava/io/OutputStream;",
                    Self::open_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
                JavaMethodProto::new(
                    "openDataOutputStream",
                    "(Ljava/lang/String;)Ljava/io/DataOutputStream;",
                    Self::open_data_output_stream,
                    MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                ),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC,
        }
    }

    // No protocol is supported yet, so every open fails the way the specification requires for an unsupported protocol.
    async fn open(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_with_mode(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>, mode: i32) -> Result<ClassInstanceRef<Connection>> {
        tracing::warn!("stub javax.microedition.io.Connector::open({name:?}, {mode})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

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

    async fn open_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<InputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_data_input_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataInputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataInputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<OutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }

    async fn open_data_output_stream(jvm: &Jvm, _: &mut WieJvmContext, name: ClassInstanceRef<String>) -> Result<ClassInstanceRef<DataOutputStream>> {
        tracing::warn!("stub javax.microedition.io.Connector::openDataOutputStream({name:?})");

        Err(jvm
            .exception("javax/microedition/io/ConnectionNotFoundException", "Unsupported protocol")
            .await)
    }
}
