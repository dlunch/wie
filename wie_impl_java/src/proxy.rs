use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::Deref,
};

use jvm::{ClassInstanceRef, JavaValue};

use crate::{base::JavaContext, method::TypeConverter};

pub struct Array<T>(PhantomData<T>);

pub struct JvmClassInstanceProxy<T> {
    pub instance: Option<ClassInstanceRef>,
    _phantom: PhantomData<T>,
}

impl<T> JvmClassInstanceProxy<T> {
    pub fn new(instance: Option<ClassInstanceRef>) -> Self {
        Self {
            instance,
            _phantom: PhantomData,
        }
    }
}

impl<T> TypeConverter<JvmClassInstanceProxy<T>> for JvmClassInstanceProxy<T> {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> JvmClassInstanceProxy<T> {
        JvmClassInstanceProxy::new(raw.as_object())
    }

    fn from_rust(_: &mut dyn JavaContext, value: JvmClassInstanceProxy<T>) -> JavaValue {
        JavaValue::Object(value.instance)
    }
}

impl<T> Debug for JvmClassInstanceProxy<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(x) = &self.instance {
            write!(f, "{:?}", x.borrow())
        } else {
            write!(f, "null")
        }
    }
}

impl<T> Deref for JvmClassInstanceProxy<T> {
    type Target = ClassInstanceRef;
    fn deref(&self) -> &Self::Target {
        self.instance.as_ref().unwrap()
    }
}
