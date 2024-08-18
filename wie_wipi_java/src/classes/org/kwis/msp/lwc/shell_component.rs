use alloc::vec;

use crate::WIPIJavaClassProto;

// class org.kwis.msp.lwc.ShellComponent
pub struct ShellComponent {}

impl ShellComponent {
    pub fn as_proto() -> WIPIJavaClassProto {
        WIPIJavaClassProto {
            name: "org/kwis/msp/lwc/ShellComponent",
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
