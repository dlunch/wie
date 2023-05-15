use alloc::vec;

use crate::base::JavaClassProto;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener {}

impl JletEventListener {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
