use std::any::Any;

pub mod org;

pub struct JavaClassImpl {
    pub name: String,
    pub methods: Vec<JavaMethodImpl>,
}

pub struct JavaMethodImpl {
    pub name: String,
    pub signature: String,
    pub body: JavaMethodBody,
}

pub type JavaMethodBody = Box<dyn Fn(Vec<Box<dyn Any>>) -> Box<dyn Any>>;

pub fn get_java_impl(name: &str) -> JavaClassImpl {
    match name {
        "org/kwis/msp/lcdui/Jlet" => org::kwis::msp::lcdui::Jlet::as_java_impl(),
        "org/kwis/msp/lcdui/Card" => org::kwis::msp::lcdui::Card::as_java_impl(),
        "org/kwis/msp/lcdui/JletEventListener" => org::kwis::msp::lcdui::JletEventListener::as_java_impl(),
        "org/kwis/msp/lcdui/Display" => org::kwis::msp::lcdui::Display::as_java_impl(),
        _ => panic!("Unknown java class: {}", name),
    }
}
