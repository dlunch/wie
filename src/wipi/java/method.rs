use std::marker::PhantomData;

use super::JavaBridge;

pub trait JavaMethodBody<E> {
    fn call(&self, java_bridge: &mut dyn JavaBridge, args: Vec<u32>) -> Result<u32, E>;
}

struct JavaMethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

impl<F, R, E> JavaMethodBody<E> for JavaMethodHolder<F, R, ()>
where
    F: Fn(&mut dyn JavaBridge) -> Result<R, E>,
    R: JavaTypeConverter<R>,
{
    fn call(&self, java_bridge: &mut dyn JavaBridge, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(java_bridge)?;

        Ok(R::from_rust(java_bridge, result))
    }
}

impl<F, R, E, P0> JavaMethodBody<E> for JavaMethodHolder<F, R, (P0,)>
where
    F: Fn(&mut dyn JavaBridge, P0) -> Result<R, E>,
    R: JavaTypeConverter<R>,
    P0: JavaTypeConverter<P0>,
{
    fn call(&self, java_bridge: &mut dyn JavaBridge, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(java_bridge, args[0]);

        let result = self.0(java_bridge, p0)?;

        Ok(R::from_rust(java_bridge, result))
    }
}

pub trait JavaMethodImpl<F, E, R, P> {
    fn into_body(self) -> Box<dyn JavaMethodBody<E>>;
}

impl<F, E, R> JavaMethodImpl<F, E, R, ()> for F
where
    F: Fn(&mut dyn JavaBridge) -> Result<R, E> + 'static,
    R: JavaTypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody<E>> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

impl<F, E, R, P1> JavaMethodImpl<F, E, R, (P1,)> for F
where
    F: Fn(&mut dyn JavaBridge, P1) -> Result<R, E> + 'static,
    R: JavaTypeConverter<R> + 'static,
    P1: JavaTypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn JavaMethodBody<E>> {
        Box::new(JavaMethodHolder(self, PhantomData))
    }
}

pub trait JavaTypeConverter<T> {
    fn to_rust(java_bridge: &mut dyn JavaBridge, raw: u32) -> T;
    fn from_rust(java_bridge: &mut dyn JavaBridge, rust: T) -> u32;
}

impl JavaTypeConverter<u32> for u32 {
    fn to_rust(_: &mut dyn JavaBridge, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut dyn JavaBridge, rust: u32) -> u32 {
        rust
    }
}

impl JavaTypeConverter<()> for () {
    fn to_rust(_: &mut dyn JavaBridge, _: u32) {}

    fn from_rust(_: &mut dyn JavaBridge, _: ()) -> u32 {
        0
    }
}
