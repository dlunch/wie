use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.RuntimeException
pub struct RuntimeException {}

impl RuntimeException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
