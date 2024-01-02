use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.Math
pub struct Math {}

impl Math {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
