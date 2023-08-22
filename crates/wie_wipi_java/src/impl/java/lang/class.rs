use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.Class
pub struct Class {}

impl Class {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
