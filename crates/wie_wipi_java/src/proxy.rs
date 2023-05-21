use crate::{base::JavaContext, method::TypeConverter};

pub struct JavaObjectProxy {
    pub ptr_instance: u32,
}

impl JavaObjectProxy {
    pub fn new(ptr_instance: u32) -> Self {
        Self { ptr_instance }
    }
}

impl TypeConverter<JavaObjectProxy> for JavaObjectProxy {
    fn to_rust(_: &mut dyn JavaContext, raw: u32) -> JavaObjectProxy {
        JavaObjectProxy::new(raw)
    }

    fn from_rust(_: &mut dyn JavaContext, rust: JavaObjectProxy) -> u32 {
        rust.ptr_instance
    }
}
