use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::Deref,
};

use jvm::{ClassInstanceRef, JavaValue};

use crate::{base::JavaContext, method::TypeConverter};

pub struct Array<T>(PhantomData<T>);

pub struct JvmClassInstanceProxy<T> {
    instance: Option<ClassInstanceRef>,
    _phantom: PhantomData<T>,
}

impl<T> TypeConverter<JvmClassInstanceProxy<T>> for JvmClassInstanceProxy<T> {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> JvmClassInstanceProxy<T> {
        Self {
            instance: raw.into(),
            _phantom: PhantomData,
        }
    }

    fn from_rust(_: &mut dyn JavaContext, value: JvmClassInstanceProxy<T>) -> JavaValue {
        value.instance.into()
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

impl<T> From<JvmClassInstanceProxy<T>> for JavaValue {
    fn from(value: JvmClassInstanceProxy<T>) -> Self {
        value.instance.into()
    }
}

impl<T> From<ClassInstanceRef> for JvmClassInstanceProxy<T> {
    fn from(value: ClassInstanceRef) -> Self {
        Self {
            instance: Some(value),
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Option<ClassInstanceRef>> for JvmClassInstanceProxy<T> {
    fn from(value: Option<ClassInstanceRef>) -> Self {
        Self {
            instance: value,
            _phantom: PhantomData,
        }
    }
}
