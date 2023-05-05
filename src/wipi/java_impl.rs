mod java;
mod org;

pub struct JavaClassProto {
    pub methods: Vec<JavaMethodProto>,
}

pub struct JavaMethodProto {
    pub name: String,
    pub signature: String,
    pub body: Box<dyn JavaMethodImpl>,
}

impl JavaMethodProto {
    pub fn new<F>(name: &str, signature: &str, body: F) -> Self
    where
        F: Fn(&mut dyn Jvm, Vec<u32>) -> u32 + 'static,
    {
        Self {
            name: name.into(),
            signature: signature.into(),
            body: Box::new(JavaMethodHolder(body)),
        }
    }
}

pub trait JavaMethodImpl {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

struct JavaMethodHolder<F>(F);

pub trait JavaMethodCaller<T> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

impl<F> JavaMethodImpl for JavaMethodHolder<F>
where
    F: Fn(&mut dyn Jvm, Vec<u32>) -> u32,
{
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32 {
        self.0(jvm, args)
    }
}

pub trait Jvm {
    fn instantiate(&mut self, class_name: &str) -> anyhow::Result<u32>;
    fn call_method(&mut self, ptr_instance: u32, name: &str, signature: &str, args: &[u32]) -> anyhow::Result<u32>;
    fn get_field(&mut self, ptr_instance: u32, field_offset: u32) -> anyhow::Result<u32>;
    fn put_field(&mut self, ptr_instance: u32, field_offset: u32, value: u32);
}

pub fn get_java_impl(name: &str) -> Option<JavaClassProto> {
    Some(match name {
        "java/lang/Exception" => java::lang::Exception::as_proto(),
        "java/lang/InterruptedException" => java::lang::InterruptedException::as_proto(),
        "java/lang/Object" => java::lang::Object::as_proto(),
        "java/lang/Runnable" => java::lang::Runnable::as_proto(),
        "java/lang/String" => java::lang::String::as_proto(),
        "java/lang/Thread" => java::lang::Thread::as_proto(),
        "org/kwis/msp/lcdui/Card" => org::kwis::msp::lcdui::Card::as_proto(),
        "org/kwis/msp/lcdui/Display" => org::kwis::msp::lcdui::Display::as_proto(),
        "org/kwis/msp/lcdui/Image" => org::kwis::msp::lcdui::Image::as_proto(),
        "org/kwis/msp/lcdui/Jlet" => org::kwis::msp::lcdui::Jlet::as_proto(),
        "org/kwis/msp/lcdui/JletEventListener" => org::kwis::msp::lcdui::JletEventListener::as_proto(),
        "org/kwis/msp/media/Clip" => org::kwis::msp::media::Clip::as_proto(),
        _ => {
            log::error!("Unknown java class: {}", name);

            return None;
        }
    })
}
