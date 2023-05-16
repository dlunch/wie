use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.System
pub struct System {}

impl System {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
