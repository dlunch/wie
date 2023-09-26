use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent {}

impl ContainerComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
