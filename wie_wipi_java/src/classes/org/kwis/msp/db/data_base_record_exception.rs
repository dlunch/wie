use alloc::vec;

use crate::WIPIJavaClassProto;

// class org.kwis.msp.db.DataBaseRecordException
pub struct DataBaseRecordException {}

impl DataBaseRecordException {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
