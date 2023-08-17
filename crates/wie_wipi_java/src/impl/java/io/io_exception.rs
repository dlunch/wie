use alloc::vec;

use crate::base::JavaClassProto;

// class java.io.IOException
pub struct IOException {}

impl IOException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
