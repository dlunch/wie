use core::marker::PhantomData;

use crate::{base::JavaContext, method::TypeConverter};

pub struct JavaObjectProxy<T> {
    pub ptr_instance: u32,
    _phantom: PhantomData<T>,
}

impl<T> JavaObjectProxy<T> {
    pub fn new(ptr_instance: u32) -> Self {
        Self {
            ptr_instance,
            _phantom: PhantomData,
        }
    }

    pub fn cast<U>(&self) -> JavaObjectProxy<U> {
        JavaObjectProxy::new(self.ptr_instance)
    }
}

impl<T> TypeConverter<JavaObjectProxy<T>> for JavaObjectProxy<T> {
    fn to_rust(_: &mut dyn JavaContext, raw: u32) -> JavaObjectProxy<T> {
        JavaObjectProxy::new(raw)
    }

    fn from_rust(_: &mut dyn JavaContext, rust: JavaObjectProxy<T>) -> u32 {
        rust.ptr_instance
    }
}
