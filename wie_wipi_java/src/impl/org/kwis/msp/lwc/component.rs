use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.lwc.Component
pub struct Component {}

impl Component {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
