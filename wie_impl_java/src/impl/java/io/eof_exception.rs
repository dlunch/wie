use alloc::vec;

use crate::base::JavaClassProto;

// class java.io.EOFException
pub struct EOFException {}

impl EOFException {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/io/IOException"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
