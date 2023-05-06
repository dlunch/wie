use std::marker::PhantomData;

pub trait MethodBody<E, C>
where
    C: ?Sized,
{
    fn call(&self, context: &mut C, args: Vec<u32>) -> Result<u32, E>;
}

struct MethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

impl<F, R, E, C> MethodBody<E, C> for MethodHolder<F, R, ()>
where
    C: ?Sized,
    F: Fn(&mut C) -> Result<R, E>,
    R: TypeConverter<R, C>,
{
    fn call(&self, context: &mut C, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(context)?;

        Ok(R::from_rust(context, result))
    }
}

impl<F, R, E, C, P0> MethodBody<E, C> for MethodHolder<F, R, (P0,)>
where
    C: ?Sized,
    F: Fn(&mut C, P0) -> Result<R, E>,
    R: TypeConverter<R, C>,
    P0: TypeConverter<P0, C>,
{
    fn call(&self, context: &mut C, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(context, args[0]);

        let result = self.0(context, p0)?;

        Ok(R::from_rust(context, result))
    }
}

pub trait MethodImpl<F, R, E, C, P>
where
    C: ?Sized,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>>;
}

impl<F, R, E, C> MethodImpl<F, R, E, C, ()> for F
where
    C: ?Sized,
    F: Fn(&mut C) -> Result<R, E> + 'static,
    R: TypeConverter<R, C> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, C, P1> MethodImpl<F, R, E, C, (P1,)> for F
where
    C: ?Sized,
    F: Fn(&mut C, P1) -> Result<R, E> + 'static,
    R: TypeConverter<R, C> + 'static,
    P1: TypeConverter<P1, C> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

pub trait TypeConverter<T, C>
where
    C: ?Sized,
{
    fn to_rust(context: &mut C, raw: u32) -> T;
    fn from_rust(context: &mut C, rust: T) -> u32;
}
