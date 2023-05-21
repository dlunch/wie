use alloc::boxed::Box;
use core::marker::PhantomData;

use crate::CContext;

pub trait MethodBody<E> {
    fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E>;
}

struct MethodHolder<F, R, P>(pub F, PhantomData<(R, P)>);

impl<F, R, E> MethodBody<E> for MethodHolder<F, R, ()>
where
    F: Fn(&mut dyn CContext) -> Result<R, E>,
    R: TypeConverter<R>,
{
    fn call(&self, context: &mut dyn CContext, _: &[u32]) -> Result<u32, E> {
        let result = self.0(context)?;

        Ok(R::from_rust(context, result))
    }
}

impl<F, R, E, P0> MethodBody<E> for MethodHolder<F, R, (P0,)>
where
    F: Fn(&mut dyn CContext, P0) -> Result<R, E>,
    R: TypeConverter<R>,
    P0: TypeConverter<P0>,
{
    fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let p0 = P0::to_rust(context, args[0]);

        let result = self.0(context, p0)?;

        Ok(R::from_rust(context, result))
    }
}

impl<F, R, E, P0, P1> MethodBody<E> for MethodHolder<F, R, (P0, P1)>
where
    F: Fn(&mut dyn CContext, P0, P1) -> Result<R, E>,
    R: TypeConverter<R>,
    P0: TypeConverter<P0>,
    P1: TypeConverter<P1>,
{
    fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);

        let result = self.0(context, p0, p1)?;

        Ok(R::from_rust(context, result))
    }
}

impl<F, R, E, P0, P1, P2> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2)>
where
    F: Fn(&mut dyn CContext, P0, P1, P2) -> Result<R, E>,
    R: TypeConverter<R>,
    P0: TypeConverter<P0>,
    P1: TypeConverter<P1>,
    P2: TypeConverter<P2>,
{
    fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);

        let result = self.0(context, p0, p1, p2)?;

        Ok(R::from_rust(context, result))
    }
}

pub trait MethodImpl<F, R, E, P> {
    fn into_body(self) -> Box<dyn MethodBody<E>>;
}

impl<F, R, E> MethodImpl<F, R, E, ()> for F
where
    F: Fn(&mut dyn CContext) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0> MethodImpl<F, R, E, (P0,)> for F
where
    F: Fn(&mut dyn CContext, P0) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
    P0: TypeConverter<P0> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1> MethodImpl<F, R, E, (P0, P1)> for F
where
    F: Fn(&mut dyn CContext, P0, P1) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
    P0: TypeConverter<P0> + 'static,
    P1: TypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2> MethodImpl<F, R, E, (P0, P1, P2)> for F
where
    F: Fn(&mut dyn CContext, P0, P1, P2) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
    P0: TypeConverter<P0> + 'static,
    P1: TypeConverter<P1> + 'static,
    P2: TypeConverter<P2> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

pub trait TypeConverter<T> {
    fn to_rust(context: &mut dyn CContext, raw: u32) -> T;
    fn from_rust(context: &mut dyn CContext, rust: T) -> u32;
}
