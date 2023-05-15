use alloc::vec;

use crate::base::JavaClassProto;

// interface java.lang.Runnable
pub struct Runnable {}

impl Runnable {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            methods: vec![],
            fields: vec![],
        }
    }
}
