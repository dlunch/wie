use std::marker::PhantomData;

pub trait MethodBody<E, C> {
    fn call(&self, context: C, args: Vec<u32>) -> Result<u32, E>;
}

struct MethodHolder<F, R, P>(pub F, std::marker::PhantomData<(R, P)>);

impl<F, R, E, C> MethodBody<E, C> for MethodHolder<F, R, ()>
where
    F: Fn(C) -> Result<R, E>,
    R: TypeConverter<R>,
{
    fn call(&self, context: C, _: Vec<u32>) -> Result<u32, E> {
        let result = self.0(context)?;

        Ok(R::from_rust(result))
    }
}

impl<F, R, E, C, P0> MethodBody<E, C> for MethodHolder<F, R, (P0,)>
where
    F: Fn(C, P0) -> Result<R, E>,
    R: TypeConverter<R>,
    P0: TypeConverter<P0>,
{
    fn call(&self, context: C, args: Vec<u32>) -> Result<u32, E> {
        let p0 = P0::to_rust(args[0]);

        let result = self.0(context, p0)?;

        Ok(R::from_rust(result))
    }
}

pub trait MethodImpl<F, R, E, C, P> {
    fn into_body(self) -> Box<dyn MethodBody<E, C>>;
}

impl<F, R, E, C> MethodImpl<F, R, E, C, ()> for F
where
    F: Fn(C) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

impl<F, R, E, C, P1> MethodImpl<F, R, E, C, (P1,)> for F
where
    F: Fn(C, P1) -> Result<R, E> + 'static,
    R: TypeConverter<R> + 'static,
    P1: TypeConverter<P1> + 'static,
{
    fn into_body(self) -> Box<dyn MethodBody<E, C>> {
        Box::new(MethodHolder(self, PhantomData))
    }
}

pub trait TypeConverter<T> {
    fn to_rust(raw: u32) -> T;
    fn from_rust(rust: T) -> u32;
}

impl TypeConverter<u32> for u32 {
    fn to_rust(raw: u32) -> u32 {
        raw
    }

    fn from_rust(rust: u32) -> u32 {
        rust
    }
}

impl TypeConverter<()> for () {
    fn to_rust(_: u32) {}

    fn from_rust(_: ()) -> u32 {
        0
    }
}
