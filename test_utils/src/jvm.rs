use core::ops::{Deref, DerefMut};

use alloc::boxed::Box;

use java_class_proto::JavaClassProto;
use jvm::{Class, Jvm, JvmResult};

use jvm_rust::{ClassImpl, JvmDetailImpl};

fn get_class_loader<T, C, Context>(get_class_proto: T, context: Context) -> impl Fn(&str) -> JvmResult<Option<Box<dyn Class>>>
where
    T: Fn(&str) -> Option<JavaClassProto<C>>,
    C: ?Sized + 'static,
    Context: DerefMut + Deref<Target = C> + Clone + 'static,
{
    move |class_name| Ok(get_class_proto(class_name).map(|x| Box::new(ClassImpl::from_class_proto(class_name, x, context.clone())) as _))
}

pub fn test_jvm<T, C, Context>(get_class_proto: T, context: Context) -> Jvm
where
    T: Fn(&str) -> Option<JavaClassProto<C>> + 'static,
    C: ?Sized + 'static,
    Context: DerefMut + Deref<Target = C> + Clone + 'static,
{
    Jvm::new(JvmDetailImpl::new(get_class_loader(get_class_proto, context)))
}
