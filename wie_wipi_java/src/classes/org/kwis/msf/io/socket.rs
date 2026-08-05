use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msf.io.Socket
pub struct Socket;

impl Socket {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msf/io/Socket",
            parent_class: None,
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new_abstract("accept", "()Lorg/kwis/msf/io/Socket;", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("close", "()V", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getInputStream", "()Ljava/io/InputStream;", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getMessageCount", "()I", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getMessageMaxLength", "()I", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("getOutputStream", "()Ljava/io/OutputStream;", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("isStream", "()Z", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("recv", "(Lorg/kwis/msf/io/Message;)V", MethodAccessFlags::ABSTRACT),
                JavaMethodProto::new_abstract("send", "(Lorg/kwis/msf/io/Message;)V", MethodAccessFlags::ABSTRACT),
            ],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
