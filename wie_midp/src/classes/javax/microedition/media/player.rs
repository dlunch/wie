use alloc::vec;

use java_class_proto::JavaMethodProto;

use wie_jvm_support::WieJavaClassProto;

// interface javax.microedition.media.Player
pub struct Player;

impl Player {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/Player",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("start", "()V", Default::default())],
            fields: vec![],
        }
    }
}
