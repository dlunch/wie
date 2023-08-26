use alloc::vec;

use crate::base::JavaClassProto;

// interface java.lang.Runnable
pub struct Runnable {}

impl Runnable {
    // TODO Create JavaInterfaceProto
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: "",
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
