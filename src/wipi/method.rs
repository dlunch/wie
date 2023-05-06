use std::marker::PhantomData;

pub trait MethodBody<E, C> {
    fn call(&self, context: C, args: Vec<u32>) -> Result<u32, E>;
}

struct MethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

impl<F, R, E, C> MethodBody<E, C> for MethodHolder<F, R, ()>
where
    F: Fn(&mut C) -> Result<R, E>,
    R: TypeConverter<R, C>,
{
    fn call(&self, mut context: C, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(&mut context)?;

        Ok(R::from_rust(&mut context, result))
    }
}

impl<F, R, E, C, P0> MethodBody<E, C> for MethodHolder<F, R, (P0,)>
where
    F: Fn(&mut C, P0) -> Result<R, E>,
    R: TypeConverter<R, C>,
    P0: TypeConverter<P0, C>,
{
    fn call(&self, mut context: C, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(&mut context, args[0]);

        let result = self.0(&mut context, p0)?;

        Ok(R::from_rust(&mut context, result))
    }
}

pub trait MethodImpl<F, R, E, C, P> {
    fn into_body(self) -> Box<dyn MethodBody<E, C>>;
}

impl<F, R, E, C> MethodImpl<F, R, E, C, ()> for F
where
    F: Fn(&mut C) -> Result<R, E> + 'static,
    R: TypeConverter<R, C> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, C, P1> MethodImpl<F, R, E, C, (P1,)> for F
where
    F: Fn(&mut C, P1) -> Result<R, E> + 'static,
    R: TypeConverter<R, C> + 'static,
    P1: TypeConverter<P1, C> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

pub trait TypeConverter<T, C> {
    fn to_rust(context: &mut C, raw: u32) -> T;
    fn from_rust(context: &mut C, rust: T) -> u32;
}
