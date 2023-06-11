use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData};

use crate::CContext;

#[async_trait::async_trait(?Send)]
pub trait MethodBody<E> {
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E>;
}

trait FnHelper<'a, E, R, P> {
    type Output: Future<Output = Result<R, E>> + 'a;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Self::Output;
}

impl<'a, E, R, F, Fut> FnHelper<'a, E, R, ()> for F
where
    F: Fn(&'a mut dyn CContext) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, _args: &[u32]) -> Fut {
        self(context)
    }
}

impl<'a, E, R, F, Fut, P0> FnHelper<'a, E, R, (P0,)> for F
where
    F: Fn(&'a mut dyn CContext, P0) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        self(context, p0)
    }
}

impl<'a, E, R, F, Fut, P0, P1> FnHelper<'a, E, R, (P0, P1)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);

        self(context, p0, p1)
    }
}

impl<'a, E, R, F, Fut, P0, P1, P2> FnHelper<'a, E, R, (P0, P1, P2)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1, P2) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
    P2: TypeConverter<P2> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);

        self(context, p0, p1, p2)
    }
}

impl<'a, E, R, F, Fut, P0, P1, P2, P3> FnHelper<'a, E, R, (P0, P1, P2, P3)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1, P2, P3) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
    P2: TypeConverter<P2> + 'a,
    P3: TypeConverter<P3> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);
        let p3 = P3::to_rust(context, args[3]);

        self(context, p0, p1, p2, p3)
    }
}

impl<'a, E, R, F, Fut, P0, P1, P2, P3, P4> FnHelper<'a, E, R, (P0, P1, P2, P3, P4)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1, P2, P3, P4) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
    P2: TypeConverter<P2> + 'a,
    P3: TypeConverter<P3> + 'a,
    P4: TypeConverter<P4> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);
        let p3 = P3::to_rust(context, args[3]);
        let p4 = P4::to_rust(context, args[4]);

        self(context, p0, p1, p2, p3, p4)
    }
}

impl<'a, E, R, F, Fut, P0, P1, P2, P3, P4, P5> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1, P2, P3, P4, P5) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
    P2: TypeConverter<P2> + 'a,
    P3: TypeConverter<P3> + 'a,
    P4: TypeConverter<P4> + 'a,
    P5: TypeConverter<P5> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);
        let p3 = P3::to_rust(context, args[3]);
        let p4 = P4::to_rust(context, args[4]);
        let p5 = P5::to_rust(context, args[5]);

        self(context, p0, p1, p2, p3, p4, p5)
    }
}

impl<'a, E, R, F, Fut, P0, P1, P2, P3, P4, P5, P6, P7, P8> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5, P6, P7, P8)> for F
where
    F: Fn(&'a mut dyn CContext, P0, P1, P2, P3, P4, P5, P6, P7, P8) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    P0: TypeConverter<P0> + 'a,
    P1: TypeConverter<P1> + 'a,
    P2: TypeConverter<P2> + 'a,
    P3: TypeConverter<P3> + 'a,
    P4: TypeConverter<P4> + 'a,
    P5: TypeConverter<P5> + 'a,
    P6: TypeConverter<P6> + 'a,
    P7: TypeConverter<P7> + 'a,
    P8: TypeConverter<P8> + 'a,
{
    type Output = Fut;
    fn do_call(&self, context: &'a mut dyn CContext, args: &[u32]) -> Fut {
        let p0 = P0::to_rust(context, args[0]);
        let p1 = P1::to_rust(context, args[1]);
        let p2 = P2::to_rust(context, args[2]);
        let p3 = P3::to_rust(context, args[3]);
        let p4 = P4::to_rust(context, args[4]);
        let p5 = P5::to_rust(context, args[5]);
        let p6 = P6::to_rust(context, args[6]);
        let p7 = P7::to_rust(context, args[7]);
        let p8 = P8::to_rust(context, args[8]);

        self(context, p0, p1, p2, p3, p4, p5, p6, p7, p8)
    }
}

struct MethodHolder<F, R, P>(pub F, PhantomData<(R, P)>);

#[async_trait::async_trait(?Send)]
impl<F, R, E> MethodBody<E> for MethodHolder<F, R, ()>
where
    F: for<'a> FnHelper<'a, E, R, ()>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0> MethodBody<E> for MethodHolder<F, R, (P0,)>
where
    F: for<'a> FnHelper<'a, E, R, (P0,)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1> MethodBody<E> for MethodHolder<F, R, (P0, P1)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1, P2> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1, P2, P3> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2, P3)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1, P2, P3, P4> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2, P3, P4)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1, P2, P3, P4, P5> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2, P3, P4, P5)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

#[async_trait::async_trait(?Send)]
impl<F, R, E, P0, P1, P2, P3, P4, P5, P6, P7, P8> MethodBody<E> for MethodHolder<F, R, (P0, P1, P2, P3, P4, P5, P6, P7, P8)>
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5, P6, P7, P8)>,
    R: TypeConverter<R>,
{
    async fn call(&self, context: &mut dyn CContext, args: &[u32]) -> Result<u32, E> {
        let result = self.0.do_call(context, args).await?;

        Ok(R::from_rust(context, result))
    }
}

pub trait MethodImpl<F, R, E, P> {
    fn into_body(self) -> Box<dyn MethodBody<E>>;
}

impl<F, R, E> MethodImpl<F, R, E, ()> for F
where
    F: for<'a> FnHelper<'a, E, R, ()> + 'static,
    R: TypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0> MethodImpl<F, R, E, (P0,)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0,)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1> MethodImpl<F, R, E, (P0, P1)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2> MethodImpl<F, R, E, (P0, P1, P2)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
    P2: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2, P3> MethodImpl<F, R, E, (P0, P1, P2, P3)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
    P2: 'static,
    P3: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2, P3, P4> MethodImpl<F, R, E, (P0, P1, P2, P3, P4)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
    P2: 'static,
    P3: 'static,
    P4: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2, P3, P4, P5> MethodImpl<F, R, E, (P0, P1, P2, P3, P4, P5)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
    P2: 'static,
    P3: 'static,
    P4: 'static,
    P5: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, P0, P1, P2, P3, P4, P5, P6, P7, P8> MethodImpl<F, R, E, (P0, P1, P2, P3, P4, P5, P6, P7, P8)> for F
where
    F: for<'a> FnHelper<'a, E, R, (P0, P1, P2, P3, P4, P5, P6, P7, P8)> + 'static,
    R: TypeConverter<R> + 'static,
    P0: 'static,
    P1: 'static,
    P2: 'static,
    P3: 'static,
    P4: 'static,
    P5: 'static,
    P6: 'static,
    P7: 'static,
    P8: 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

pub trait TypeConverter<T> {
    fn to_rust(context: &mut dyn CContext, raw: u32) -> T;
    fn from_rust(context: &mut dyn CContext, rust: T) -> u32;
}
