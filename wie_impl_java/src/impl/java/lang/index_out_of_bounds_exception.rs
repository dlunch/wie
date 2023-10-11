use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.IndexOutOfBoundsException
pub struct IndexOutOfBoundsException {}

impl IndexOutOfBoundsException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/RuntimeException"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
