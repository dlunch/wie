use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.db.DataBaseRecordException
pub struct DataBaseRecordException {}

impl DataBaseRecordException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
