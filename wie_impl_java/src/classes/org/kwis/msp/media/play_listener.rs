use alloc::vec;

use crate::JavaClassProto;

// interface org.kwis.msp.media.PlayListener
pub struct PlayListener {}

impl PlayListener {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
