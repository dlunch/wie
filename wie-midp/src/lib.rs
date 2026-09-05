#![no_std]
extern crate alloc;

pub mod classes;

use wie_jvm_support::WieJavaClassProto;

pub fn get_protos() -> [WieJavaClassProto; 36] {
    [
        classes::javax::microedition::lcdui::Alert::as_proto(),
        classes::javax::microedition::lcdui::AlertType::as_proto(),
        classes::javax::microedition::lcdui::Canvas::as_proto(),
        classes::javax::microedition::lcdui::Choice::as_proto(),
        classes::javax::microedition::lcdui::ChoiceGroup::as_proto(),
        classes::javax::microedition::lcdui::Command::as_proto(),
        classes::javax::microedition::lcdui::CommandListener::as_proto(),
        classes::javax::microedition::lcdui::Display::as_proto(),
        classes::javax::microedition::lcdui::Displayable::as_proto(),
        classes::javax::microedition::lcdui::Font::as_proto(),
        classes::javax::microedition::lcdui::Form::as_proto(),
        classes::javax::microedition::lcdui::Gauge::as_proto(),
        classes::javax::microedition::lcdui::Graphics::as_proto(),
        classes::javax::microedition::lcdui::Image::as_proto(),
        classes::javax::microedition::lcdui::ImageItem::as_proto(),
        classes::javax::microedition::lcdui::Item::as_proto(),
        classes::javax::microedition::lcdui::ItemCommandListener::as_proto(),
        classes::javax::microedition::lcdui::ItemStateListener::as_proto(),
        classes::javax::microedition::lcdui::Screen::as_proto(),
        classes::javax::microedition::lcdui::StringItem::as_proto(),
        classes::javax::microedition::lcdui::TextBox::as_proto(),
        classes::javax::microedition::lcdui::Ticker::as_proto(),
        classes::javax::microedition::lcdui::game::GameCanvas::as_proto(),
        classes::javax::microedition::media::Manager::as_proto(),
        classes::javax::microedition::media::MediaException::as_proto(),
        classes::javax::microedition::media::Player::as_proto(),
        classes::javax::microedition::midlet::MIDlet::as_proto(),
        classes::javax::microedition::rms::InvalidRecordIDException::as_proto(),
        classes::javax::microedition::rms::RecordStore::as_proto(),
        classes::javax::microedition::rms::RecordStoreException::as_proto(),
        classes::net::wie::ChoiceElement::as_proto(),
        classes::net::wie::EventQueue::as_proto(),
        classes::net::wie::ItemStateEvent::as_proto(),
        classes::net::wie::Launcher::as_proto(),
        classes::net::wie::SmafPlayer::as_proto(),
        classes::net::wie::WieError::as_proto(),
    ]
}
