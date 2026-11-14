mod volume_control;

use alloc::vec;

use jvm_types::ClassAccessFlags;

use wie_jvm_support::WieJavaClassProto;

pub use self::volume_control::VolumeControl;

// interface javax.microedition.media.Control
pub struct Control;

impl Control {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/media/Control",
            parent_class: None,
            interfaces: vec![],
            methods: vec![],
            fields: vec![],
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
        }
    }
}
