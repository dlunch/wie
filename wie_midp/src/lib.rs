#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 3] {
    [
        classes::javax::microedition::lcdui::Canvas::as_proto(),
        classes::javax::microedition::lcdui::Graphics::as_proto(),
        classes::javax::microedition::midlet::MIDlet::as_proto(),
    ]
}
