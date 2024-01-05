use alloc::boxed::Box;
use core::{
    fmt::{self, Debug, Formatter},
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use jvm::{ClassInstance, JavaValue};

use crate::{base::JavaContext, method::TypeConverter};

pub struct Array<T>(PhantomData<T>);

pub struct JvmClassInstanceHandle<T> {
    instance: Option<Box<dyn ClassInstance>>,
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
            write!(f, "{:?}", x)
        } else {
            write!(f, "null")
        }
    }
}

impl<T> Deref for JvmClassInstanceHandle<T> {
    type Target = Box<dyn ClassInstance>;
    fn deref(&self) -> &Self::Target {
        self.instance.as_ref().unwrap()
    }
}

impl<T> DerefMut for JvmClassInstanceHandle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.instance.as_mut().unwrap()
    }
}

impl<T> From<JvmClassInstanceHandle<T>> for JavaValue {
    fn from(value: JvmClassInstanceHandle<T>) -> Self {
        value.instance.into()
    }
}

impl<T> From<Box<dyn ClassInstance>> for JvmClassInstanceHandle<T> {
    fn from(value: Box<dyn ClassInstance>) -> Self {
        Self {
            instance: Some(value),
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Option<Box<dyn ClassInstance>>> for JvmClassInstanceHandle<T> {
    fn from(value: Option<Box<dyn ClassInstance>>) -> Self {
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

impl<T> From<JvmClassInstanceHandle<T>> for Box<dyn ClassInstance> {
    fn from(value: JvmClassInstanceHandle<T>) -> Self {
        value.instance.unwrap()
    }
}
