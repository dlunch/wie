use super::{method::JavaTypeConverter, JavaBridge};

pub struct JavaObjectProxy {
    pub ptr_instance: u32,
}

impl JavaObjectProxy {
    pub fn new(ptr_instance: u32) -> Self {
        Self { ptr_instance }
    }
}

impl JavaTypeConverter<JavaObjectProxy> for JavaObjectProxy {
    fn to_rust(_: &mut dyn JavaBridge, raw: u32) -> JavaObjectProxy {
        JavaObjectProxy::new(raw)
    }

    fn from_rust(_: &mut dyn JavaBridge, rust: JavaObjectProxy) -> u32 {
        rust.ptr_instance
    }
}
