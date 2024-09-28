use alloc::vec;

use wie_jvm_support::WieJavaClassProto;

// class javax.microedition.lcdui.Graphics
pub struct Graphics;

impl Graphics {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/lcdui/Graphics",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
