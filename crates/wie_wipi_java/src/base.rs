use alloc::{boxed::Box, string::String, vec::Vec};

use wie_backend::Backend;
use wie_base::method::{MethodBody, MethodImpl, TypeConverter};

use crate::{array::Array, proxy::JavaObjectProxy};

pub struct JavaClassProto {
    pub methods: Vec<JavaMethodProto>,
    pub fields: Vec<JavaFieldProto>,
}

pub type JavaError = anyhow::Error;
pub type JavaResult<T> = anyhow::Result<T>;

pub type JavaContext = dyn JavaContextBase;

pub struct JavaMethodProto {
    pub name: String,
    pub signature: String,
    pub body: JavaMethodBody,
}

pub struct JavaFieldProto {
    pub name: String,
    pub signature: String,
}

impl JavaFieldProto {
    pub fn new(name: &str, signature: &str) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
        }
    }
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

pub trait JavaContextBase {
    fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy>;
    fn instantiate_array(&mut self, element_type_name: &str, count: u32) -> JavaResult<JavaObjectProxy>;
    fn call_method(&mut self, instance: &JavaObjectProxy, name: &str, signature: &str, args: &[u32]) -> JavaResult<u32>;
    fn get_field(&mut self, instance: &JavaObjectProxy, field_name: &str) -> JavaResult<u32>;
    fn put_field(&mut self, instance: &JavaObjectProxy, field_name: &str, value: u32) -> JavaResult<()>;
    fn schedule_task(&mut self, callback: JavaMethodBody) -> JavaResult<()>;
    fn backend(&mut self) -> &mut Backend;
}

pub fn get_class_proto(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/lang/Exception" => crate::r#impl::java::lang::Exception::as_proto(),
        "java/lang/InterruptedException" => crate::r#impl::java::lang::InterruptedException::as_proto(),
        "java/lang/Object" => crate::r#impl::java::lang::Object::as_proto(),
        "java/lang/Runnable" => crate::r#impl::java::lang::Runnable::as_proto(),
        "java/lang/Runtime" => crate::r#impl::java::lang::Runtime::as_proto(),
        "java/lang/String" => crate::r#impl::java::lang::String::as_proto(),
        "java/lang/Thread" => crate::r#impl::java::lang::Thread::as_proto(),
        "org/kwis/msp/handset/BackLight" => crate::r#impl::org::kwis::msp::handset::BackLight::as_proto(),
        "org/kwis/msp/lcdui/Card" => crate::r#impl::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => crate::r#impl::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/Graphics" => crate::r#impl::org::kwis::msp::lcdui::Graphics::as_proto(),
        "org/kwis/msp/lcdui/Image" => crate::r#impl::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => crate::r#impl::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => crate::r#impl::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/lwc/AnnunciatorComponent" => crate::r#impl::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        "org/kwis/msp/media/Clip" => crate::r#impl::org::kwis::msp::media::Clip::as_proto(),

        _ => return None,
    })
}

pub fn get_array_proto() -> JavaClassProto {
    Array::as_proto()
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
