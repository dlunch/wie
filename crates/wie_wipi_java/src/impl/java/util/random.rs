use alloc::vec;

use crate::base::JavaClassProto;

// class java.util.Random
pub struct Random {}

impl Random {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/util/Random"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
