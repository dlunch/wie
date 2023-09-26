use alloc::vec;

use crate::base::JavaClassProto;

// class org.kwis.msp.lwc.TextFieldComponent
pub struct TextFieldComponent {}

impl TextFieldComponent {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("org/kwis/msp/lwc/TextComponent"),
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
        }
    }
}
