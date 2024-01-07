#![no_std]
extern crate alloc;

pub mod org;

use alloc::boxed::Box;

use dyn_clone::{clone_trait_object, DynClone};

use java_runtime_base::{JavaResult, MethodBody};

use wie_backend::SystemHandle;

pub trait JavaContext: DynClone {
    fn system(&mut self) -> &mut SystemHandle;
    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()>;
}

clone_trait_object!(JavaContext);

pub(crate) type JavaClassProto = java_runtime_base::JavaClassProto<dyn JavaContext>;
pub(crate) type JavaMethodBody = Box<dyn MethodBody<anyhow::Error, JavaContextArg>>;
pub(crate) type JavaContextArg = dyn JavaContext;

pub fn get_class_proto(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "org/kwis/msf/io/Network" => crate::org::kwis::msf::io::Network::as_proto(),
        "org/kwis/msp/db/DataBase" => crate::org::kwis::msp::db::DataBase::as_proto(),
        "org/kwis/msp/db/DataBaseException" => crate::org::kwis::msp::db::DataBaseException::as_proto(),
        "org/kwis/msp/db/DataBaseRecordException" => crate::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        "org/kwis/msp/handset/BackLight" => crate::org::kwis::msp::handset::BackLight::as_proto(),
        "org/kwis/msp/handset/HandsetProperty" => crate::org::kwis::msp::handset::HandsetProperty::as_proto(),
        "org/kwis/msp/io/File" => crate::org::kwis::msp::io::File::as_proto(),
        "org/kwis/msp/io/FileSystem" => crate::org::kwis::msp::io::FileSystem::as_proto(),
        "org/kwis/msp/lcdui/Card" => crate::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => crate::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/EventQueue" => crate::org::kwis::msp::lcdui::EventQueue::as_proto(),
        "org/kwis/msp/lcdui/Font" => crate::org::kwis::msp::lcdui::Font::as_proto(),
        "org/kwis/msp/lcdui/Graphics" => crate::org::kwis::msp::lcdui::Graphics::as_proto(),
        "org/kwis/msp/lcdui/Image" => crate::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Main" => crate::org::kwis::msp::lcdui::Main::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => crate::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => crate::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/lwc/AnnunciatorComponent" => crate::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        "org/kwis/msp/lwc/Component" => crate::org::kwis::msp::lwc::Component::as_proto(),
        "org/kwis/msp/lwc/ContainerComponent" => crate::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        "org/kwis/msp/lwc/ShellComponent" => crate::org::kwis::msp::lwc::ShellComponent::as_proto(),
        "org/kwis/msp/lwc/TextComponent" => crate::org::kwis::msp::lwc::TextComponent::as_proto(),
        "org/kwis/msp/lwc/TextFieldComponent" => crate::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        "org/kwis/msp/media/Clip" => crate::org::kwis::msp::media::Clip::as_proto(),
        "org/kwis/msp/media/Player" => crate::org::kwis::msp::media::Player::as_proto(),
        "org/kwis/msp/media/PlayListener" => crate::org::kwis::msp::media::PlayListener::as_proto(),
        "org/kwis/msp/media/Vibrator" => crate::org::kwis::msp::media::Vibrator::as_proto(),

        _ => return None,
    })
}
