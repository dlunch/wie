use alloc::vec;

use crate::base::JavaClassProto;

// class java.util.Date
pub struct Date {}

impl Date {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
