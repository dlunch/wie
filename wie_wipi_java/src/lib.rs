#![no_std]
extern crate alloc;

pub mod classes;
mod context;

use core::future::Future;

use context::WIPIJavaClassProto;
pub use context::WIPIJavaContext;

use alloc::boxed::Box;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

// TODO we need class loader
pub async fn register<T, F>(jvm: &Jvm, class_creator: T) -> JvmResult<()>
where
    T: Fn(WIPIJavaClassProto) -> F,
    F: Future<Output = Box<dyn ClassDefinition>>,
{
    // superclass should come before subclass
    let protos = [
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
    ];

    for proto in protos {
        let class = class_creator(proto).await;

        jvm.register_class(class, None).await?;
    }

    Ok(())
}
