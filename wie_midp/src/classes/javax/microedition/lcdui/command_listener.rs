use alloc::vec;

use java_class_proto::JavaMethodProto;

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.lcdui.CommandListener
pub struct CommandListener;

impl CommandListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/CommandListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract(
                "commandAction",
                "(Ljavax/microedition/lcdui/Command;Ljavax/microedition/lcdui/Displayable;)V",
                Default::default(),
            )],
            fields: vec![],
        }
    }
}
