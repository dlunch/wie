use alloc::vec;

use crate::WieClassProto;

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent {}

impl ContainerComponent {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
