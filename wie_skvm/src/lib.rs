#![no_std]
extern crate alloc;

use wie_jvm_support::WieJavaClassProto;

pub mod classes;

pub fn get_protos() -> [WieJavaClassProto; 7] {
    [
        classes::com::skt::m::AudioClip::as_proto(),
        classes::com::skt::m::AudioSystem::as_proto(),
        classes::com::skt::m::BackLight::as_proto(),
        classes::com::skt::m::Device::as_proto(),
        classes::com::skt::m::Graphics2D::as_proto(),
        classes::com::xce::io::XFile::as_proto(),
        classes::com::xce::lcdui::Toolkit::as_proto(),
    ]
}
