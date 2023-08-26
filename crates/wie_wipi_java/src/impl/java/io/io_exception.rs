use alloc::vec;

use crate::base::JavaClassProto;

// class java.io.IOException
pub struct IOException {}

impl IOException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: "java/lang/Exception",
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
