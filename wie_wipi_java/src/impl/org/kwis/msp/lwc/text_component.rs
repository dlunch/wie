use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.lwc.TextComponent
pub struct TextComponent {}

impl TextComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/Component"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
