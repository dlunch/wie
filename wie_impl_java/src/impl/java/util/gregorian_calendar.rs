use alloc::vec;

use crate::base::JavaClassProto;

// class java.util.GregorianCalendar
pub struct GregorianCalendar {}

impl GregorianCalendar {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/util/Calendar"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
