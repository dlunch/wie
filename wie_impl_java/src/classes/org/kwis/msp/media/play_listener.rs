use alloc::vec;

use crate::WieClassProto;

// interface org.kwis.msp.media.PlayListener
pub struct PlayListener {}

impl PlayListener {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
