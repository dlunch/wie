use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_constants::ClassAccessFlags;
use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lwc.EventListener
pub struct EventListener;

impl EventListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lwc/EventListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract(
                "eventNotify",
                "(IIIILjava/lang/Object;)Z",
                Default::default(),
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::INTERFACE,
        }
    }
}
