#![no_std]
extern crate alloc;

use wie_jvm_support::WieJavaClassProto;

pub mod classes;

pub fn get_protos() -> [WieJavaClassProto; 26] {
    [
        classes::com::skt::m::AudioClip::as_proto(),
        classes::com::skt::m::AudioSystem::as_proto(),
        classes::com::skt::m::BackLight::as_proto(),
        classes::com::skt::m::Call::as_proto(),
        classes::com::skt::m::Device::as_proto(),
        classes::com::skt::m::Graphics2D::as_proto(),
        classes::com::skt::m::MathFP::as_proto(),
        classes::com::skt::m::PhoneBook::as_proto(),
        classes::com::skt::m::ProgressBar::as_proto(),
        classes::com::skt::m::ResourceAllocException::as_proto(),
        classes::com::skt::m::SISImage::as_proto(),
        classes::com::skt::m::SMS::as_proto(),
        classes::com::skt::m::SMSListener::as_proto(),
        classes::com::skt::m::SMSMessage::as_proto(),
        classes::com::skt::m::UnsupportedFormatException::as_proto(),
        classes::com::skt::m::UserStopException::as_proto(),
        classes::com::skt::m::Vibration::as_proto(),
        classes::com::skt::m3d::Graphics3D::as_proto(),
        classes::com::skt::m3d::Object3D::as_proto(),
        classes::com::xce::io::FileInputStream::as_proto(),
        classes::com::xce::io::FileOutputStream::as_proto(),
        classes::com::xce::io::XFile::as_proto(),
        classes::com::xce::lcdui::Toolkit::as_proto(),
        classes::com::xce::lcdui::XDisplay::as_proto(),
        classes::com::xce::lcdui::XTextField::as_proto(),
        classes::net::wie::WieAudioClip::as_proto(),
    ]
}
