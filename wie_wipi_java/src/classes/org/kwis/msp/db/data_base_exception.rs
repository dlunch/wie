use alloc::vec;

use crate::context::WIPIJavaClassProto;

// class org.kwis.msp.db.DataBaseException
pub struct DataBaseException {}

impl DataBaseException {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            name: "org/kwis/msp/db/DataBaseException",
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
