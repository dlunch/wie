use crate::wipi::method::TypeConverter;

use super::JavaContext;

pub struct JavaObjectProxy {
    pub ptr_instance: u32,
}

impl JavaObjectProxy {
    pub fn new(ptr_instance: u32) -> Self {
        Self { ptr_instance }
    }
}

impl TypeConverter<JavaObjectProxy, JavaContext> for JavaObjectProxy {
    fn to_rust(_: &mut JavaContext, raw: u32) -> JavaObjectProxy {
        JavaObjectProxy::new(raw)
    }

    fn from_rust(_: &mut JavaContext, rust: JavaObjectProxy) -> u32 {
        rust.ptr_instance
    }
}
