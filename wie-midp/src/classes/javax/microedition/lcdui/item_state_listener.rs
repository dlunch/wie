use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.lcdui.ItemStateListener
pub struct ItemStateListener;

impl ItemStateListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/ItemStateListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract(
                "itemStateChanged",
                "(Ljavax/microedition/lcdui/Item;)V",
                MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
            )],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
