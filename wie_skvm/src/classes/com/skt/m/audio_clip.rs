use alloc::vec;

use java_class_proto::JavaMethodProto;

use wie_jvm_support::WieJavaClassProto;

// interface com.skt.m.AudioClip
pub struct AudioClip {}

impl AudioClip {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/skt/m/AudioClip",
            parent_class: None,
            interfaces: vec![],
            methods: vec![JavaMethodProto::new_abstract("play", "()V", Default::default())],
            fields: vec![],
        }
    }
}
