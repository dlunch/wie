use core::marker::PhantomData;

use jvm::ClassInstanceRef;

use crate::{
    base::{JavaContext, JavaWord},
    method::TypeConverter,
};

pub struct JavaObjectProxy<T> {
    pub ptr_instance: JavaWord,
    _phantom: PhantomData<T>,
}

impl<T> JavaObjectProxy<T> {
    pub fn new(ptr_instance: JavaWord) -> Self {
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
    fn to_rust(_: &mut dyn JavaContext, raw: usize) -> JavaObjectProxy<T> {
        JavaObjectProxy::new(raw as _)
    }

    fn from_rust(_: &mut dyn JavaContext, rust: JavaObjectProxy<T>) -> usize {
        rust.ptr_instance as _
    }
}

pub struct JvmClassInstanceProxy<T> {
    pub class_instance: ClassInstanceRef,
    _phantom: PhantomData<T>,
}

impl<T> TypeConverter<JvmClassInstanceProxy<T>> for JvmClassInstanceProxy<T> {
    fn to_rust(context: &mut dyn JavaContext, raw: JavaWord) -> JvmClassInstanceProxy<T> {
        JvmClassInstanceProxy {
            class_instance: context.instance_from_raw(raw),
            _phantom: PhantomData,
        }
    }

    fn from_rust(context: &mut dyn JavaContext, value: JvmClassInstanceProxy<T>) -> JavaWord {
        context.instance_raw(&value.class_instance)
    }
}
