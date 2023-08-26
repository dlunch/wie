use alloc::vec;

use crate::base::JavaClassProto;

// class java.lang.Exception
pub struct Exception {}

impl Exception {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: "java/lang/Throwable",
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
