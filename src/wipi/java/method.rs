use std::marker::PhantomData;

use super::Jvm;

pub trait JavaMethodBody<E> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> Result<u32, E>;
}

struct JavaMethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

trait JavaMethodCaller<T, E> {
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> Result<u32, E>;
}

impl<F, R, E> JavaMethodBody<E> for JavaMethodHolder<F, R, ()>
where
    F: Fn(&mut dyn Jvm) -> Result<R, E>,
    R: JavaTypeConverter<R>,
{
    fn call(&self, jvm: &mut dyn Jvm, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(jvm)?;

        Ok(R::from_rust(jvm, result))
    }
}

impl<F, R, E, P0> JavaMethodBody<E> for JavaMethodHolder<F, R, (P0,)>
where
    F: Fn(&mut dyn Jvm, P0) -> Result<R, E>,
    R: JavaTypeConverter<R>,
    P0: JavaTypeConverter<P0>,
{
    fn call(&self, jvm: &mut dyn Jvm, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(jvm, args[0]);

        let result = self.0(jvm, p0)?;

        Ok(R::from_rust(jvm, result))
    }
}

pub trait JavaMethodImpl<F, E, R, P> {
    fn into_body(self) -> Box<dyn JavaMethodBody<E>>;
}

impl<F, E, R> JavaMethodImpl<F, E, R, ()> for F
where
    F: Fn(&mut dyn Jvm) -> Result<R, E> + 'static,
    R: JavaTypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody<E>> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

impl<F, E, R, P1> JavaMethodImpl<F, E, R, (P1,)> for F
where
    F: Fn(&mut dyn Jvm, P1) -> Result<R, E> + 'static,
    R: JavaTypeConverter<R> + 'static,
    P1: JavaTypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody<E>> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

pub trait JavaTypeConverter<T> {
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
