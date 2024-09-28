use alloc::vec;

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.media.PlayListener
pub struct PlayListener;

impl PlayListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/media/PlayListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
