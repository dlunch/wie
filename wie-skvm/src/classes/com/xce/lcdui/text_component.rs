use alloc::vec;

use jvm_class_proto::JavaMethodProto;
use jvm_types::{ClassAccessFlags, MethodAccessFlags};

use wie_jvm_support::WieJavaClassProto;

// interface com.xce.lcdui.TextComponent
pub struct TextComponent;

impl TextComponent {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/TextComponent",
            parent_class: None,
            interfaces: vec![],
            methods: [
                ("clear", "()V"),
                ("delete", "()V"),
                ("getCaretPosition", "()I"),
                ("getConstraints", "()I"),
                ("getMaxSize", "()I"),
                ("insert", "(C)V"),
                ("moveCursor", "(I)V"),
                ("repaint", "()V"),
                ("repaintIM", "()V"),
                ("replace", "(C)V"),
                ("setCaretPosition", "(I)V"),
                ("setCaretVisible", "(Z)V"),
                ("size", "()I"),
            ]
            .into_iter()
            .map(|(name, descriptor)| JavaMethodProto::new_abstract(name, descriptor, MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT))
            .collect(),
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
