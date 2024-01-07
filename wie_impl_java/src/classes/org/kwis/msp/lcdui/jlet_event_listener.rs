use alloc::vec;

use crate::WieClassProto;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener {}

impl JletEventListener {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
