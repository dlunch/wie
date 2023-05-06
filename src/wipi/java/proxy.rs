use crate::wipi::method::TypeConverter;

pub struct JavaObjectProxy {
    pub ptr_instance: u32,
}

impl JavaObjectProxy {
    pub fn new(ptr_instance: u32) -> Self {
        Self { ptr_instance }
    }
}

impl TypeConverter<JavaObjectProxy> for JavaObjectProxy {
    fn to_rust(raw: u32) -> JavaObjectProxy {
        JavaObjectProxy::new(raw)
    }

    fn from_rust(rust: JavaObjectProxy) -> u32 {
        rust.ptr_instance
    }
}
