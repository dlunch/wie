mod r#impl;
mod method;
mod proxy;

use method::JavaMethodImpl;
pub use {method::JavaMethodBody, proxy::JavaObjectProxy};

pub struct JavaClassProto {
    pub methods: Vec<JavaMethodProto>,
}

pub type JavaError = anyhow::Error;
pub type JavaResult<T> = anyhow::Result<T>;

pub struct JavaMethodProto {
    pub name: String,
    pub signature: String,
    pub body: Box<dyn JavaMethodBody<JavaError>>,
}

impl JavaMethodProto {
    pub fn new<M, F, R, P>(name: &str, signature: &str, method: M) -> Self
    where
        M: JavaMethodImpl<F, JavaError, R, P>,
    {
        Self {
            name: name.into(),
            signature: signature.into(),
            body: method.into_body(),
        }
    }
}

pub trait Jvm {
    fn instantiate(&mut self, class_name: &str) -> JavaResult<JavaObjectProxy>;
    fn call_method(&mut self, instance: &JavaObjectProxy, name: &str, signature: &str, args: &[u32]) -> JavaResult<u32>;
    fn get_field(&mut self, instance: &JavaObjectProxy, field_offset: u32) -> JavaResult<u32>;
    fn put_field(&mut self, instance: &JavaObjectProxy, field_offset: u32, value: u32);
}

pub fn get_java_impl(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/lang/Exception" => r#impl::java::lang::Exception::as_proto(),
        "java/lang/InterruptedException" => r#impl::java::lang::InterruptedException::as_proto(),
        "java/lang/Object" => r#impl::java::lang::Object::as_proto(),
        "java/lang/Runnable" => r#impl::java::lang::Runnable::as_proto(),
        "java/lang/String" => r#impl::java::lang::String::as_proto(),
        "java/lang/Thread" => r#impl::java::lang::Thread::as_proto(),
        "org/kwis/msp/lcdui/Card" => r#impl::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => r#impl::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/Image" => r#impl::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => r#impl::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => r#impl::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/media/Clip" => r#impl::org::kwis::msp::media::Clip::as_proto(),
        _ => {
            log::error!("Unknown java class: {}", name);

            return None;
        }
    })
}
