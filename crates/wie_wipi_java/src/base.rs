use alloc::{boxed::Box, string::String, vec::Vec};

use wie_backend::{task::SleepFuture, Backend};

use crate::{
    array::Array,
    method::{MethodBody, MethodImpl, TypeConverter},
    proxy::JavaObjectProxy,
};

pub struct JavaClassProto {
    pub methods: Vec<JavaMethodProto>,
    pub fields: Vec<JavaFieldProto>,
}

pub type JavaError = anyhow::Error;
pub type JavaResult<T> = anyhow::Result<T>;

#[derive(Eq, PartialEq)]
pub enum JavaMethodAccessFlag {
    NONE,
    STATIC = 0x8,
    NATIVE = 0x100,
}

#[derive(Eq, PartialEq)]
pub enum JavaFieldAccessFlag {
    NONE,
    STATIC = 0x8,
}

pub struct JavaMethodProto {
    pub name: String,
    pub signature: String,
    pub body: JavaMethodBody,
    pub access_flag: JavaMethodAccessFlag,
}

pub struct JavaFieldProto {
    pub name: String,
    pub signature: String,
    pub access_flag: JavaFieldAccessFlag,
}

impl JavaFieldProto {
    pub fn new(name: &str, signature: &str, access_flag: JavaFieldAccessFlag) -> Self {
        Self {
            name: name.into(),
            signature: signature.into(),
            access_flag,
        }
    }
}

pub type JavaMethodBody = Box<dyn MethodBody<JavaError>>;

impl JavaMethodProto {
    pub fn new<M, F, R, P>(name: &str, signature: &str, method: M, access_flag: JavaMethodAccessFlag) -> Self
    where
        M: MethodImpl<F, R, JavaError, P>,
    {
        Self {
            name: name.into(),
            signature: signature.into(),
            body: method.into_body(),
            access_flag,
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait JavaContext {
    fn instantiate(&mut self, type_name: &str) -> JavaResult<JavaObjectProxy>;
    fn instantiate_array(&mut self, element_type_name: &str, count: u32) -> JavaResult<JavaObjectProxy>;
    fn destroy(&mut self, instance: JavaObjectProxy) -> JavaResult<()>;
    async fn call_method(&mut self, instance: &JavaObjectProxy, name: &str, signature: &str, args: &[u32]) -> JavaResult<u32>;
    async fn call_static_method(&mut self, class_name: &str, method_name: &str, signature: &str, args: &[u32]) -> JavaResult<u32>;
    fn get_field(&mut self, instance: &JavaObjectProxy, field_name: &str) -> JavaResult<u32>;
    fn put_field(&mut self, instance: &JavaObjectProxy, field_name: &str, value: u32) -> JavaResult<()>;
    fn get_static_field(&mut self, class_name: &str, field_name: &str) -> JavaResult<u32>;
    fn put_static_field(&mut self, class_name: &str, field_name: &str, value: u32) -> JavaResult<()>;
    fn store_array(&mut self, array: &JavaObjectProxy, offset: u32, values: &[u32]) -> JavaResult<()>;
    fn load_array(&mut self, array: &JavaObjectProxy, offset: u32, count: u32) -> JavaResult<Vec<u32>>;
    fn array_length(&mut self, array: &JavaObjectProxy) -> JavaResult<u32>;
    fn backend(&mut self) -> &mut Backend;
    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()>;
    fn sleep(&mut self, duration: u64) -> SleepFuture;
}

pub fn get_class_proto(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/lang/Exception" => crate::r#impl::java::lang::Exception::as_proto(),
        "java/lang/IllegalArgumentException" => crate::r#impl::java::lang::IllegalArgumentException::as_proto(),
        "java/lang/InterruptedException" => crate::r#impl::java::lang::InterruptedException::as_proto(),
        "java/lang/Object" => crate::r#impl::java::lang::Object::as_proto(),
        "java/lang/Runnable" => crate::r#impl::java::lang::Runnable::as_proto(),
        "java/lang/Runtime" => crate::r#impl::java::lang::Runtime::as_proto(),
        "java/lang/String" => crate::r#impl::java::lang::String::as_proto(),
        "java/lang/StringBuffer" => crate::r#impl::java::lang::StringBuffer::as_proto(),
        "java/lang/System" => crate::r#impl::java::lang::System::as_proto(),
        "java/lang/Thread" => crate::r#impl::java::lang::Thread::as_proto(),
        "java/io/IOException" => crate::r#impl::java::io::IOException::as_proto(),
        "org/kwis/msp/db/DataBase" => crate::r#impl::org::kwis::msp::db::DataBase::as_proto(),
        "org/kwis/msp/db/DataBaseRecordException" => crate::r#impl::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        "org/kwis/msp/handset/BackLight" => crate::r#impl::org::kwis::msp::handset::BackLight::as_proto(),
        "org/kwis/msp/handset/HandsetProperty" => crate::r#impl::org::kwis::msp::handset::HandsetProperty::as_proto(),
        "org/kwis/msp/lcdui/Card" => crate::r#impl::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => crate::r#impl::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/Font" => crate::r#impl::org::kwis::msp::lcdui::Font::as_proto(),
        "org/kwis/msp/lcdui/Graphics" => crate::r#impl::org::kwis::msp::lcdui::Graphics::as_proto(),
        "org/kwis/msp/lcdui/Image" => crate::r#impl::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => crate::r#impl::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => crate::r#impl::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/lwc/AnnunciatorComponent" => crate::r#impl::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        "org/kwis/msp/media/Clip" => crate::r#impl::org::kwis::msp::media::Clip::as_proto(),
        "org/kwis/msp/media/PlayListener" => crate::r#impl::org::kwis::msp::media::PlayListener::as_proto(),

        _ => return None,
    })
}

pub fn get_array_proto() -> JavaClassProto {
    Array::as_proto()
}

impl TypeConverter<u32> for u32 {
    fn to_rust(_: &mut dyn JavaContext, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut dyn JavaContext, rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: &mut dyn JavaContext, _: u32) {}

    fn from_rust(_: &mut dyn JavaContext, _: ()) -> u32 {
        0
    }
}
