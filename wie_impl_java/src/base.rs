use alloc::{boxed::Box, string::String, vec::Vec};

use jvm::{JavaValue, Jvm};

use wie_backend::{task::SleepFuture, Backend};

use crate::method::{MethodBody, MethodImpl, TypeConverter};

pub struct JavaClassProto {
    pub parent_class: Option<&'static str>,
    pub interfaces: Vec<&'static str>,
    pub methods: Vec<JavaMethodProto>,
    pub fields: Vec<JavaFieldProto>,
}

pub type JavaError = anyhow::Error;
pub type JavaResult<T> = anyhow::Result<T>;
pub type JavaWord = usize;

#[derive(Eq, PartialEq)]
pub enum JavaMethodFlag {
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
    pub descriptor: String,
    pub body: JavaMethodBody,
    pub flag: JavaMethodFlag,
}

pub struct JavaFieldProto {
    pub name: String,
    pub descriptor: String,
    pub access_flag: JavaFieldAccessFlag,
}

impl JavaFieldProto {
    pub fn new(name: &str, descriptor: &str, access_flag: JavaFieldAccessFlag) -> Self {
        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            access_flag,
        }
    }
}

pub type JavaMethodBody = Box<dyn MethodBody<JavaError>>;

impl JavaMethodProto {
    pub fn new<M, F, R, P>(name: &str, descriptor: &str, method: M, flag: JavaMethodFlag) -> Self
    where
        M: MethodImpl<F, R, JavaError, P>,
    {
        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            body: method.into_body(),
            flag,
        }
    }

    pub fn new_abstract(name: &str, descriptor: &str, flag: JavaMethodFlag) -> Self {
        struct AbstractCall {
            name: String,
            descriptor: String,
        }

        #[async_trait::async_trait(?Send)]
        impl MethodBody<JavaError> for AbstractCall {
            async fn call(&self, _: &mut dyn JavaContext, _: Box<[JavaValue]>) -> Result<JavaValue, JavaError> {
                // TODO throw java.lang.AbstractMethodError
                anyhow::bail!("Call to abstract function {}{}", self.name, self.descriptor)
            }
        }

        Self {
            name: name.into(),
            descriptor: descriptor.into(),
            body: Box::new(AbstractCall {
                name: name.into(),
                descriptor: descriptor.into(),
            }),
            flag,
        }
    }
}

#[async_trait::async_trait(?Send)]
pub trait JavaContext {
    fn jvm(&mut self) -> &mut Jvm;
    fn backend(&mut self) -> &mut Backend;
    fn spawn(&mut self, callback: JavaMethodBody) -> JavaResult<()>;
    fn sleep(&mut self, duration: u64) -> SleepFuture;
}

pub fn get_class_proto(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/io/ByteArrayInputStream" => crate::r#impl::java::io::ByteArrayInputStream::as_proto(),
        "java/io/DataInputStream" => crate::r#impl::java::io::DataInputStream::as_proto(),
        "java/io/EOFException" => crate::r#impl::java::io::EOFException::as_proto(),
        "java/io/IOException" => crate::r#impl::java::io::IOException::as_proto(),
        "java/io/InputStream" => crate::r#impl::java::io::InputStream::as_proto(),
        "java/io/OutputStream" => crate::r#impl::java::io::OutputStream::as_proto(),
        "java/io/PrintStream" => crate::r#impl::java::io::PrintStream::as_proto(),
        "java/lang/Class" => crate::r#impl::java::lang::Class::as_proto(),
        "java/lang/Exception" => crate::r#impl::java::lang::Exception::as_proto(),
        "java/lang/IllegalArgumentException" => crate::r#impl::java::lang::IllegalArgumentException::as_proto(),
        "java/lang/IndexOutOfBoundsException" => crate::r#impl::java::lang::IndexOutOfBoundsException::as_proto(),
        "java/lang/Integer" => crate::r#impl::java::lang::Integer::as_proto(),
        "java/lang/InterruptedException" => crate::r#impl::java::lang::InterruptedException::as_proto(),
        "java/lang/NullPointerException" => crate::r#impl::java::lang::NullPointerException::as_proto(),
        "java/lang/Object" => crate::r#impl::java::lang::Object::as_proto(),
        "java/lang/Runnable" => crate::r#impl::java::lang::Runnable::as_proto(),
        "java/lang/Runtime" => crate::r#impl::java::lang::Runtime::as_proto(),
        "java/lang/RuntimeException" => crate::r#impl::java::lang::RuntimeException::as_proto(),
        "java/lang/SecurityException" => crate::r#impl::java::lang::SecurityException::as_proto(),
        "java/lang/String" => crate::r#impl::java::lang::String::as_proto(),
        "java/lang/StringBuffer" => crate::r#impl::java::lang::StringBuffer::as_proto(),
        "java/lang/System" => crate::r#impl::java::lang::System::as_proto(),
        "java/lang/Thread" => crate::r#impl::java::lang::Thread::as_proto(),
        "java/lang/Throwable" => crate::r#impl::java::lang::Throwable::as_proto(),
        "java/util/Calendar" => crate::r#impl::java::util::Calendar::as_proto(),
        "java/util/Date" => crate::r#impl::java::util::Date::as_proto(),
        "java/util/GregorianCalendar" => crate::r#impl::java::util::GregorianCalendar::as_proto(),
        "java/util/Hashtable" => crate::r#impl::java::util::Hashtable::as_proto(),
        "java/util/Random" => crate::r#impl::java::util::Random::as_proto(),
        "java/util/Timer" => crate::r#impl::java::util::Timer::as_proto(),
        "java/util/TimerTask" => crate::r#impl::java::util::TimerTask::as_proto(),
        "java/util/Vector" => crate::r#impl::java::util::Vector::as_proto(),
        "org/kwis/msf/io/Network" => crate::r#impl::org::kwis::msf::io::Network::as_proto(),
        "org/kwis/msp/db/DataBase" => crate::r#impl::org::kwis::msp::db::DataBase::as_proto(),
        "org/kwis/msp/db/DataBaseException" => crate::r#impl::org::kwis::msp::db::DataBaseException::as_proto(),
        "org/kwis/msp/db/DataBaseRecordException" => crate::r#impl::org::kwis::msp::db::DataBaseRecordException::as_proto(),
        "org/kwis/msp/handset/BackLight" => crate::r#impl::org::kwis::msp::handset::BackLight::as_proto(),
        "org/kwis/msp/handset/HandsetProperty" => crate::r#impl::org::kwis::msp::handset::HandsetProperty::as_proto(),
        "org/kwis/msp/io/File" => crate::r#impl::org::kwis::msp::io::File::as_proto(),
        "org/kwis/msp/io/FileSystem" => crate::r#impl::org::kwis::msp::io::FileSystem::as_proto(),
        "org/kwis/msp/lcdui/Card" => crate::r#impl::org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => crate::r#impl::org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/EventQueue" => crate::r#impl::org::kwis::msp::lcdui::EventQueue::as_proto(),
        "org/kwis/msp/lcdui/Font" => crate::r#impl::org::kwis::msp::lcdui::Font::as_proto(),
        "org/kwis/msp/lcdui/Graphics" => crate::r#impl::org::kwis::msp::lcdui::Graphics::as_proto(),
        "org/kwis/msp/lcdui/Image" => crate::r#impl::org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Main" => crate::r#impl::org::kwis::msp::lcdui::Main::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => crate::r#impl::org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => crate::r#impl::org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/lwc/AnnunciatorComponent" => crate::r#impl::org::kwis::msp::lwc::AnnunciatorComponent::as_proto(),
        "org/kwis/msp/lwc/Component" => crate::r#impl::org::kwis::msp::lwc::Component::as_proto(),
        "org/kwis/msp/lwc/ContainerComponent" => crate::r#impl::org::kwis::msp::lwc::ContainerComponent::as_proto(),
        "org/kwis/msp/lwc/ShellComponent" => crate::r#impl::org::kwis::msp::lwc::ShellComponent::as_proto(),
        "org/kwis/msp/lwc/TextComponent" => crate::r#impl::org::kwis::msp::lwc::TextComponent::as_proto(),
        "org/kwis/msp/lwc/TextFieldComponent" => crate::r#impl::org::kwis::msp::lwc::TextFieldComponent::as_proto(),
        "org/kwis/msp/media/Clip" => crate::r#impl::org::kwis::msp::media::Clip::as_proto(),
        "org/kwis/msp/media/Player" => crate::r#impl::org::kwis::msp::media::Player::as_proto(),
        "org/kwis/msp/media/PlayListener" => crate::r#impl::org::kwis::msp::media::PlayListener::as_proto(),
        "org/kwis/msp/media/Vibrator" => crate::r#impl::org::kwis::msp::media::Vibrator::as_proto(),

        _ => return None,
    })
}

impl TypeConverter<i32> for i32 {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> i32 {
        raw.as_int()
    }

    fn from_rust(_: &mut dyn JavaContext, rust: i32) -> JavaValue {
        JavaValue::Int(rust)
    }
}

impl TypeConverter<i64> for i64 {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> i64 {
        raw.as_long()
    }

    fn from_rust(_: &mut dyn JavaContext, rust: i64) -> JavaValue {
        JavaValue::Long(rust)
    }
}

impl TypeConverter<bool> for bool {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> bool {
        raw.as_boolean()
    }

    fn from_rust(_: &mut dyn JavaContext, rust: bool) -> JavaValue {
        JavaValue::Boolean(rust)
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: &mut dyn JavaContext, _: JavaValue) {}

    fn from_rust(_: &mut dyn JavaContext, _: ()) -> JavaValue {
        JavaValue::Void
    }
}
