#![no_std]
extern crate alloc;

use wie_jvm_support::WieJavaClassProto;

pub mod classes;

pub fn get_protos() -> [WieJavaClassProto; 16] {
    [
        classes::com::skt::m::AudioClip::as_proto(),
        classes::com::skt::m::AudioSystem::as_proto(),
        classes::com::skt::m::BackLight::as_proto(),
        classes::com::skt::m::Device::as_proto(),
        classes::com::skt::m::Graphics2D::as_proto(),
        classes::com::skt::m::MathFP::as_proto(),
        classes::com::skt::m::Vibration::as_proto(),
        classes::com::skt::m::ProgressBar::as_proto(),
        classes::com::skt::m::UnsupportedFormatException::as_proto(),
        classes::com::xce::io::FileInputStream::as_proto(),
        classes::com::xce::io::FileOutputStream::as_proto(),
        classes::com::xce::io::XFile::as_proto(),
        classes::com::xce::lcdui::Toolkit::as_proto(),
        classes::com::xce::lcdui::XDisplay::as_proto(),
        classes::com::xce::lcdui::XTextField::as_proto(),
        classes::net::wie::WieAudioClip::as_proto(),
    ]
}
