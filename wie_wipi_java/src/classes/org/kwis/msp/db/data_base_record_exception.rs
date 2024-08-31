use alloc::vec;

use wie_jvm_support::WieJavaClassProto;

// class org.kwis.msp.db.DataBaseRecordException
pub struct DataBaseRecordException {}

impl DataBaseRecordException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "org/kwis/msp/db/DataBaseRecordException",
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
