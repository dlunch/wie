use alloc::vec;

use crate::WieClassProto;

// class org.kwis.msp.db.DataBaseException
pub struct DataBaseException {}

impl DataBaseException {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
