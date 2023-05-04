use std::any::Any;

mod java;
mod org;

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

pub trait Jvm {
    fn instantiate(&mut self, class_name: &str) -> anyhow::Result<u32>;
    fn call_method(&mut self, ptr_instance: u32, name: &str, signature: &str, args: &[u32]) -> anyhow::Result<u32>;
    fn get_field(&mut self, ptr_instance: u32, field_offset: u32) -> anyhow::Result<u32>;
    fn put_field(&mut self, ptr_instance: u32, field_offset: u32, value: u32);
}

pub fn get_java_impl(name: &str) -> Option<JavaClassImpl> {
    Some(match name {
        "java/lang/Exception" => java::lang::Exception::as_java_impl(),
        "java/lang/InterruptedException" => java::lang::InterruptedException::as_java_impl(),
        "java/lang/Object" => java::lang::Object::as_java_impl(),
        "java/lang/Runnable" => java::lang::Runnable::as_java_impl(),
        "java/lang/String" => java::lang::String::as_java_impl(),
        "java/lang/Thread" => java::lang::Thread::as_java_impl(),
        "org/kwis/msp/lcdui/Card" => org::kwis::msp::lcdui::Card::as_java_impl(),
        "org/kwis/msp/lcdui/Display" => org::kwis::msp::lcdui::Display::as_java_impl(),
        "org/kwis/msp/lcdui/Image" => org::kwis::msp::lcdui::Image::as_java_impl(),
        "org/kwis/msp/lcdui/Jlet" => org::kwis::msp::lcdui::Jlet::as_java_impl(),
        "org/kwis/msp/lcdui/JletEventListener" => org::kwis::msp::lcdui::JletEventListener::as_java_impl(),
        "org/kwis/msp/media/Clip" => org::kwis::msp::media::Clip::as_java_impl(),
        _ => {
            log::error!("Unknown java class: {}", name);

            return None;
        }
    })
}
