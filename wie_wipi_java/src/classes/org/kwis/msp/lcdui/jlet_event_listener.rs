use alloc::vec;

use crate::context::WIPIJavaClassProto;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener {}

impl JletEventListener {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
