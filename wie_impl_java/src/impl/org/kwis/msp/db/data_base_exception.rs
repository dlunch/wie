use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.db.DataBaseException
pub struct DataBaseException {}

impl DataBaseException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
