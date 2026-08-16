use alloc::vec;

use java_constants::ClassAccessFlags;

use wie_jvm_support::WieJavaClassProto;

// abstract class org.kwis.msp.lcdui.JletWrapper
pub struct JletWrapper;

impl JletWrapper {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/JletWrapper",
            parent_class: Some("org/kwis/msp/lcdui/Jlet"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::ABSTRACT,
        }
    }
}
