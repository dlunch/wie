use std::marker::PhantomData;

use super::Jvm;

pub trait JavaMethodBody {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

struct JavaMethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

trait JavaMethodCaller<T> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32;
}

impl<F, R> JavaMethodBody for JavaMethodHolder<F, R, ()>
where
    F: Fn(&mut dyn Jvm) -> R,
    R: JavaTypeConverter<R>,
{
    fn call(&self, jvm: &mut dyn Jvm, _: Vec<u32>) -> u32 {
        let result = self.0(jvm);

        R::from_rust(jvm, result)
    }
}

impl<F, R, P0> JavaMethodBody for JavaMethodHolder<F, R, (P0,)>
where
    F: Fn(&mut dyn Jvm, P0) -> R,
    R: JavaTypeConverter<R>,
    P0: JavaTypeConverter<P0>,
{
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> u32 {
        let p0 = P0::to_rust(jvm, args[0]);

        let result = self.0(jvm, p0);

        R::from_rust(jvm, result)
    }
}

pub trait JavaMethodImpl<F, R, P> {
    fn into_body(self) -> Box<dyn JavaMethodBody>;
}

impl<F, R> JavaMethodImpl<F, R, ()> for F
where
    F: Fn(&mut dyn Jvm) -> R + 'static,
    R: JavaTypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

impl<F, R, P1> JavaMethodImpl<F, R, (P1,)> for F
where
    F: Fn(&mut dyn Jvm, P1) -> R + 'static,
    R: JavaTypeConverter<R> + 'static,
    P1: JavaTypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

trait JavaTypeConverter<T> {
    fn to_rust(jvm: &mut dyn Jvm, raw: u32) -> T;
    fn from_rust(jvm: &mut dyn Jvm, rust: T) -> u32;
}

impl JavaTypeConverter<u32> for u32 {
    fn to_rust(_: &mut dyn Jvm, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut dyn Jvm, rust: u32) -> u32 {
        rust
    }
}

impl JavaTypeConverter<()> for () {
    fn to_rust(_: &mut dyn Jvm, _: u32) {}

    fn from_rust(_: &mut dyn Jvm, _: ()) -> u32 {
        0
    }
}
