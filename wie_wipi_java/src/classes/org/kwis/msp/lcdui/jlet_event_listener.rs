use alloc::vec;

use wie_jvm_support::WieJavaClassProto;

// interface org.kwis.msp.lcdui.JletEventListener
pub struct JletEventListener;

impl JletEventListener {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/lcdui/JletEventListener",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
