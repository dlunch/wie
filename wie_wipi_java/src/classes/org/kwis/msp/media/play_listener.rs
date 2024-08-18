use alloc::vec;

use crate::WIPIJavaClassProto;

// interface org.kwis.msp.media.PlayListener
pub struct PlayListener {}

impl PlayListener {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            name: "org/kwis/msp/media/PlayListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
