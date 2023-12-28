use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
};

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

impl<T> JvmClassInstanceProxy<T> {
    pub fn new(class_instance: ClassInstanceRef) -> Self {
        Self {
            class_instance,
            _phantom: PhantomData,
        }
    }
}

impl<T> TypeConverter<JvmClassInstanceProxy<T>> for JvmClassInstanceProxy<T> {
    fn to_rust(context: &mut dyn JavaContext, raw: JavaWord) -> JvmClassInstanceProxy<T> {
        JvmClassInstanceProxy::new(context.instance_from_raw(raw))
    }

    fn from_rust(context: &mut dyn JavaContext, value: JvmClassInstanceProxy<T>) -> JavaWord {
        context.instance_raw(&value.class_instance)
    }
}

impl<T> Debug for JvmClassInstanceProxy<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.class_instance.borrow())
    }
}

pub struct JvmArrayClassInstanceProxy<T> {
    pub class_instance: ClassInstanceRef,
    _phantom: PhantomData<T>,
}

impl<T> JvmArrayClassInstanceProxy<T> {
    pub fn new(class_instance: ClassInstanceRef) -> Self {
        Self {
            class_instance,
            _phantom: PhantomData,
        }
    }
}

impl<T> TypeConverter<JvmArrayClassInstanceProxy<T>> for JvmArrayClassInstanceProxy<T> {
    fn to_rust(context: &mut dyn JavaContext, raw: JavaWord) -> JvmArrayClassInstanceProxy<T> {
        JvmArrayClassInstanceProxy::new(context.array_instance_from_raw(raw))
    }

    fn from_rust(context: &mut dyn JavaContext, value: JvmArrayClassInstanceProxy<T>) -> JavaWord {
        context.instance_raw(&value.class_instance)
    }
}

impl<T> Debug for JvmArrayClassInstanceProxy<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.class_instance.borrow())
    }
}
