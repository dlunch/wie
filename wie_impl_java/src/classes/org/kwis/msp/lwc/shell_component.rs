use alloc::vec;

use crate::WieClassProto;

// class org.kwis.msp.lwc.ShellComponent
pub struct ShellComponent {}

impl ShellComponent {
    pub fn as_proto() -> WieClassProto {
        WieClassProto {
            parent_class: Some("org/kwis/msp/lwc/ContainerComponent"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
