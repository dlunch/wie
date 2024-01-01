use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::Deref,
};

use jvm::{ClassInstanceRef, JavaValue};

use crate::{base::JavaContext, method::TypeConverter};

pub struct Array<T>(PhantomData<T>);

pub struct JvmClassInstanceHandle<T> {
    instance: Option<ClassInstanceRef>,
    _phantom: PhantomData<T>,
}

impl<T> JvmClassInstanceHandle<T> {
    pub fn is_null(&self) -> bool {
        self.instance.is_none()
    }
}

impl<T> TypeConverter<JvmClassInstanceHandle<T>> for JvmClassInstanceHandle<T> {
    fn to_rust(_: &mut dyn JavaContext, raw: JavaValue) -> JvmClassInstanceHandle<T> {
        Self {
            instance: raw.into(),
            _phantom: PhantomData,
        }
    }

    fn from_rust(_: &mut dyn JavaContext, value: JvmClassInstanceHandle<T>) -> JavaValue {
        value.instance.into()
    }
}

impl<T> Debug for JvmClassInstanceHandle<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if let Some(x) = &self.instance {
            write!(f, "{:?}", x.borrow())
        } else {
            write!(f, "null")
        }
    }
}

impl<T> Deref for JvmClassInstanceHandle<T> {
    type Target = ClassInstanceRef;
    fn deref(&self) -> &Self::Target {
        self.instance.as_ref().unwrap()
    }
}

impl<T> From<JvmClassInstanceHandle<T>> for JavaValue {
    fn from(value: JvmClassInstanceHandle<T>) -> Self {
        value.instance.into()
    }
}

impl<T> From<ClassInstanceRef> for JvmClassInstanceHandle<T> {
    fn from(value: ClassInstanceRef) -> Self {
        Self {
            instance: Some(value),
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Option<ClassInstanceRef>> for JvmClassInstanceHandle<T> {
    fn from(value: Option<ClassInstanceRef>) -> Self {
        Self {
            instance: value,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<JavaValue> for JvmClassInstanceHandle<T> {
    fn from(val: JavaValue) -> Self {
        JvmClassInstanceHandle {
            instance: val.into(),
            _phantom: PhantomData,
        }
    }
}

impl<T> From<JvmClassInstanceHandle<T>> for ClassInstanceRef {
    fn from(value: JvmClassInstanceHandle<T>) -> Self {
        value.instance.unwrap()
    }
}
