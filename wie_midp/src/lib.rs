#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 8] {
    [
        classes::javax::microedition::lcdui::Canvas::as_proto(),
        classes::javax::microedition::lcdui::Display::as_proto(),
        classes::javax::microedition::lcdui::Displayable::as_proto(),
        classes::javax::microedition::lcdui::Font::as_proto(),
        classes::javax::microedition::lcdui::Graphics::as_proto(),
        classes::javax::microedition::lcdui::Image::as_proto(),
        classes::javax::microedition::midlet::MIDlet::as_proto(),
        classes::javax::microedition::rms::RecordStore::as_proto(),
    ]
}
