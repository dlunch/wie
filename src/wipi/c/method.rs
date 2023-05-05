use std::marker::PhantomData;

use super::CBridge;

// TODO: can we merge this and java::method?

pub trait CMethodBody<E> {
    fn call(&self, bridge: &mut dyn CBridge, args: Vec<u32>) -> Result<u32, E>;
}

struct CMethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

impl<F, R, E> CMethodBody<E> for CMethodHolder<F, R, ()>
where
    F: Fn(&mut dyn CBridge) -> Result<R, E>,
    R: CTypeConverter<R>,
{
    fn call(&self, bridge: &mut dyn CBridge, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(bridge)?;

        Ok(R::from_rust(bridge, result))
    }
}

impl<F, R, E, P0> CMethodBody<E> for CMethodHolder<F, R, (P0,)>
where
    F: Fn(&mut dyn CBridge, P0) -> Result<R, E>,
    R: CTypeConverter<R>,
    P0: CTypeConverter<P0>,
{
    fn call(&self, bridge: &mut dyn CBridge, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(bridge, args[0]);

        let result = self.0(bridge, p0)?;

        Ok(R::from_rust(bridge, result))
    }
}

pub trait CMethodImpl<F, E, R, P> {
    fn into_body(self) -> Box<dyn CMethodBody<E>>;
}

impl<F, E, R> CMethodImpl<F, E, R, ()> for F
where
    F: Fn(&mut dyn CBridge) -> Result<R, E> + 'static,
    R: CTypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn CMethodBody<E>> {
        Box::new(CMethodHolder(self, PhantomData))
    }
}

impl<F, E, R, P1> CMethodImpl<F, E, R, (P1,)> for F
where
    F: Fn(&mut dyn CBridge, P1) -> Result<R, E> + 'static,
    R: CTypeConverter<R> + 'static,
    P1: CTypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn CMethodBody<E>> {
        Box::new(CMethodHolder(self, PhantomData))
    }
}

pub trait CTypeConverter<T> {
    fn to_rust(bridge: &mut dyn CBridge, raw: u32) -> T;
    fn from_rust(bridge: &mut dyn CBridge, rust: T) -> u32;
}

impl CTypeConverter<u32> for u32 {
    fn to_rust(_: &mut dyn CBridge, raw: u32) -> u32 {
        raw
    }

    fn from_rust(_: &mut dyn CBridge, rust: u32) -> u32 {
        rust
    }
}

impl CTypeConverter<()> for () {
    fn to_rust(_: &mut dyn CBridge, _: u32) {}

    fn from_rust(_: &mut dyn CBridge, _: ()) -> u32 {
        0
    }
}
