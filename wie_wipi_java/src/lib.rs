#![no_std]

extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 28] {
    [
        crate::classes::org::kwis::msf::io::Network::as_proto(),
        crate::classes::org::kwis::msp::db::DataBase::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseException::as_proto(),
        crate::classes::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        crate::classes::org::kwis::msp::handset::BackLight::as_proto(),
        crate::classes::org::kwis::msp::handset::HandsetProperty::as_proto(),
        crate::classes::org::kwis::msp::io::File::as_proto(),
        crate::classes::org::kwis::msp::io::FileSystem::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Card::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Display::as_proto(),
        crate::classes::org::kwis::msp::lcdui::EventQueue::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Font::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Graphics::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Image::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Main::as_proto(),
        crate::classes::org::kwis::msp::lcdui::Jlet::as_proto(),
        crate::classes::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        crate::classes::org::kwis::msp::lwc::Component::as_proto(),
        crate::classes::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::ShellComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextComponent::as_proto(),
        crate::classes::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        crate::classes::org::kwis::msp::media::Clip::as_proto(),
        crate::classes::org::kwis::msp::media::Player::as_proto(),
        crate::classes::org::kwis::msp::media::PlayListener::as_proto(),
        crate::classes::org::kwis::msp::media::Vibrator::as_proto(),
        crate::classes::org::kwis::msp::media::Volume::as_proto(),
    ]
}
