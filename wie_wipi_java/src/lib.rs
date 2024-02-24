#![no_std]
extern crate alloc;

pub mod classes;
mod context;

use core::future::Future;

use context::WIPIJavaClassProto;
pub use context::WIPIJavaContextBase;

use alloc::boxed::Box;
use jvm::{ClassDefinition, Jvm, Result as JvmResult};

// TODO we need class loader
pub async fn register<T, F>(jvm: &Jvm, class_creator: T) -> JvmResult<()>
where
    T: Fn(&str, WIPIJavaClassProto) -> F,
    F: Future<Output = Box<dyn ClassDefinition>>,
{
    // superclass should come before subclass
    let classes = [
        ("org/kwis/msf/io/Network", crate::classes::org::kwis::msf::io::Network::as_proto()),
        ("org/kwis/msp/db/DataBase", crate::classes::org::kwis::msp::db::DataBase::as_proto()),
        (
            "org/kwis/msp/db/DataBaseException",
            crate::classes::org::kwis::msp::db::DataBaseException::as_proto(),
        ),
        (
            "org/kwis/msp/db/DataBaseRecordException",
            crate::classes::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        ),
        (
            "org/kwis/msp/handset/BackLight",
            crate::classes::org::kwis::msp::handset::BackLight::as_proto(),
        ),
        (
            "org/kwis/msp/handset/HandsetProperty",
            crate::classes::org::kwis::msp::handset::HandsetProperty::as_proto(),
        ),
        ("org/kwis/msp/io/File", crate::classes::org::kwis::msp::io::File::as_proto()),
        ("org/kwis/msp/io/FileSystem", crate::classes::org::kwis::msp::io::FileSystem::as_proto()),
        ("org/kwis/msp/lcdui/Card", crate::classes::org::kwis::msp::lcdui::Card::as_proto()),
        ("org/kwis/msp/lcdui/Display", crate::classes::org::kwis::msp::lcdui::Display::as_proto()),
        (
            "org/kwis/msp/lcdui/EventQueue",
            crate::classes::org::kwis::msp::lcdui::EventQueue::as_proto(),
        ),
        ("org/kwis/msp/lcdui/Font", crate::classes::org::kwis::msp::lcdui::Font::as_proto()),
        ("org/kwis/msp/lcdui/Graphics", crate::classes::org::kwis::msp::lcdui::Graphics::as_proto()),
        ("org/kwis/msp/lcdui/Image", crate::classes::org::kwis::msp::lcdui::Image::as_proto()),
        ("org/kwis/msp/lcdui/Main", crate::classes::org::kwis::msp::lcdui::Main::as_proto()),
        ("org/kwis/msp/lcdui/Jlet", crate::classes::org::kwis::msp::lcdui::Jlet::as_proto()),
        (
            "org/kwis/msp/lcdui/JletEventListener",
            crate::classes::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        ),
        ("org/kwis/msp/lwc/Component", crate::classes::org::kwis::msp::lwc::Component::as_proto()),
        (
            "org/kwis/msp/lwc/ContainerComponent",
            crate::classes::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        ),
        (
            "org/kwis/msp/lwc/ShellComponent",
            crate::classes::org::kwis::msp::lwc::ShellComponent::as_proto(),
        ),
        (
            "org/kwis/msp/lwc/AnnunciatorComponent",
            crate::classes::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        ),
        (
            "org/kwis/msp/lwc/TextComponent",
            crate::classes::org::kwis::msp::lwc::TextComponent::as_proto(),
        ),
        (
            "org/kwis/msp/lwc/TextFieldComponent",
            crate::classes::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        ),
        ("org/kwis/msp/media/Clip", crate::classes::org::kwis::msp::media::Clip::as_proto()),
        ("org/kwis/msp/media/Player", crate::classes::org::kwis::msp::media::Player::as_proto()),
        (
            "org/kwis/msp/media/PlayListener",
            crate::classes::org::kwis::msp::media::PlayListener::as_proto(),
        ),
        ("org/kwis/msp/media/Vibrator", crate::classes::org::kwis::msp::media::Vibrator::as_proto()),
    ];

    for (name, proto) in classes {
        let class = class_creator(name, proto).await;

        jvm.register_class(class, None).await?;
    }

    Ok(())
}
