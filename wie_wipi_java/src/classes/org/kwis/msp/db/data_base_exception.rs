use alloc::vec;

use wie_jvm_support::WieJavaClassProto;

// class org.kwis.msp.db.DataBaseException
pub struct DataBaseException {}

impl DataBaseException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBaseException",
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
