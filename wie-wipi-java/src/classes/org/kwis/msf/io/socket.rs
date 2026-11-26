use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};
use jvm_class_proto::{java_class, java_methods};
use rustjava_runtime::classes::java::io::{InputStream, OutputStream};

use wie_jvm_support::WieJvmContext;

use crate::classes::org::kwis::msf::io::Message;

// interface org.kwis.msf.io.Socket
#[java_class(name = "org/kwis/msf/io/Socket", public, interface, abstract)]
pub struct Socket;

#[java_methods]
impl Socket {
    #[method(public)]
    async fn accept(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<Self>>;

    #[method(public)]
    async fn close(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<()>;

    #[method(public)]
    async fn get_input_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<InputStream>>;

    #[method(public)]
    async fn get_message_count(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_message_max_length(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<i32>;

    #[method(public)]
    async fn get_output_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<ClassInstanceRef<OutputStream>>;

    #[method(public)]
    async fn is_stream(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> JvmResult<bool>;

    #[method(public)]
    async fn recv(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<Message>) -> JvmResult<()>;

    #[method(public)]
    async fn send(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<Message>) -> JvmResult<()>;
}
