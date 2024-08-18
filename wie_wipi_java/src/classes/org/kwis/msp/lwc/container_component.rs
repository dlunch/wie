use alloc::vec;

use crate::WIPIJavaClassProto;

// class org.kwis.msp.lwc.ContainerComponent
pub struct ContainerComponent {}

impl ContainerComponent {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            name: "org/kwis/msp/lwc/ContainerComponent",
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
