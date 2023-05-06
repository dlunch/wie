mod array;
mod r#impl;
mod proxy;

use super::method::{MethodBody, MethodImpl, TypeConverter};
pub use proxy::JavaObjectProxy;

pub struct JavaClassProto {
    pub methods: Vec<JavaMethodProto>,
}

pub type JavaError = anyhow::Error;
pub type JavaResult<T> = anyhow::Result<T>;

pub type JavaContext = dyn JavaBridge;

pub struct JavaMethodProto {
    pub name: String,
    pub signature: String,
    pub body: JavaMethodBody,
}

pub type JavaMethodBody = Box<dyn MethodBody<JavaError, JavaContext>>;

impl JavaMethodProto {
    pub fn new<M, F, R, P>(name: &str, signature: &str, method: M) -> Self
    where
        M: MethodImpl<F, R, JavaError, JavaContext, P>,
    {
        Self {
            name: name.into(),
            signature: signature.into(),
            body: method.into_body(),
        }
    }
}

pub trait JavaBridge {
    fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy>;
    fn instantiate_array(&mut self, element_type_name: &str, count: u32) -> JavaResult<JavaObjectProxy>;
    fn call_method(&mut self, instance: &JavaObjectProxy, name: &str, signature: &str, args: &[u32]) -> JavaResult<u32>;
    fn get_field(&mut self, instance: &JavaObjectProxy, field_offset: u32) -> JavaResult<u32>;
    fn put_field(&mut self, instance: &JavaObjectProxy, field_offset: u32, value: u32);
}

pub fn get_class_proto(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/lang/Exception" => r#impl::java::lang::Exception::as_proto(),
        "java/lang/InterruptedException" => r#impl::java::lang::InterruptedException::as_proto(),
        "java/lang/Object" => r#impl::java::lang::Object::as_proto(),
        "java/lang/Runnable" => r#impl::java::lang::Runnable::as_proto(),
        "java/lang/Runtime" => r#impl::java::lang::Runtime::as_proto(),
        "java/lang/String" => r#impl::java::lang::String::as_proto(),
        "java/lang/Thread" => r#impl::java::lang::Thread::as_proto(),
        "org/kwis/msp/lcdui/Card" => r#impl::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => r#impl::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/Graphics" => r#impl::org::kwis::msp::lcdui::Graphics::as_proto(),
        "org/kwis/msp/lcdui/Image" => r#impl::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => r#impl::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => r#impl::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/lwc/AnnunciatorComponent" => r#impl::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        "org/kwis/msp/media/Clip" => r#impl::org::kwis::msp::media::Clip::as_proto(),

        _ => return None,
    })
}

pub fn get_array_proto() -> JavaClassProto {
    array::Array::as_proto()
}

impl TypeConverter<u32, JavaContext> for u32 {
    fn to_rust(_: &mut JavaContext, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut JavaContext, rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<(), JavaContext> for () {
    fn to_rust(_: &mut JavaContext, _: u32) {}

    fn from_rust(_: &mut JavaContext, _: ()) -> u32 {
        0
    }
}
