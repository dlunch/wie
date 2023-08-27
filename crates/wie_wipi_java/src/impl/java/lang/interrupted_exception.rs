use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.InterruptedException
pub struct InterruptedException {}

impl InterruptedException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Exception"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
