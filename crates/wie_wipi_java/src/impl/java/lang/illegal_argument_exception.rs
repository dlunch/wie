use alloc::vec;

use crate::base::JavaClassProto;

// class java/lang/IllegalArgumentException
pub struct IllegalArgumentException {}

impl IllegalArgumentException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
