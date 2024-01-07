use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData};

use crate::{WIPICContext, WIPICWord};

macro_rules! __impl_fn_helper {
    ($context: ident, $raw_type: ty, $($arg: ident),*) => {
        impl<'a, E, R, F, Fut, $($arg),*> FnHelper<'a, E, R, ($($arg,)*)> for F
        where
            F: Fn(&'a mut dyn $context, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a,
            $($arg: TypeConverter<$arg> + 'a),*
        {
            type Output = Fut;
            #[allow(unused_assignments, non_snake_case, unused_mut, unused_variables)]
            fn do_call(&self, context: &'a mut dyn $context, args: Box<[$raw_type]>) -> Fut {
                let mut args = alloc::vec::Vec::from(args).into_iter();
                $(
                    let $arg = $arg::to_rust(context, args.next().unwrap());
                )*
                self(context, $($arg),*)
            }
        }
    };
}

macro_rules! __impl_method_body {
    ($context: ident, $raw_type: ty, $($arg: ident),*) => {
        #[async_trait::async_trait(?Send)]
        impl<F, R, E, $($arg),*> MethodBody<E> for MethodHolder<F, R, ($($arg,)*)>
        where
            F: for<'a> FnHelper<'a, E, R, ($($arg,)*)>,
            R: TypeConverter<R>,
        {
            async fn call(&self, context: &mut dyn $context, args: Box<[$raw_type]>) -> Result<$raw_type, E> {
                let result = self.0.do_call(context, args).await?;

                Ok(R::from_rust(context, result))
            }
        }
    };
}

macro_rules! __impl_method_impl {
    ($($arg: ident),*) => {
        impl<F, R, E, $($arg),*> MethodImpl<F, R, E, ($($arg,)*)> for F
        where
            F: for<'a> FnHelper<'a, E, R, ($($arg,)*)> + 'static,
            R: TypeConverter<R> + 'static,
            $($arg: 'static),*
        {
            fn into_body(self) -> Box<dyn MethodBody<E>> {
                Box::new(MethodHolder(self, PhantomData))
            }
        }
    };
}

macro_rules! __generate {
    ($context: ident, $raw_type: ty, $($arg: ident),*) => {
        __impl_fn_helper!($context, $raw_type, $($arg),*);
        __impl_method_body!($context, $raw_type, $($arg),*);
        __impl_method_impl!($($arg),*);
    };
}

macro_rules! methods {
    ($context: ident, $raw_type: ty) => {
        #[async_trait::async_trait(?Send)]
        pub trait MethodBody<E> {
            async fn call(&self, context: &mut dyn $context, args: Box<[$raw_type]>) -> Result<$raw_type, E>;
        }

        trait FnHelper<'a, E, R, P> {
            type Output: Future<Output = Result<R, E>> + 'a;
            fn do_call(&self, context: &'a mut dyn $context, args: Box<[$raw_type]>) -> Self::Output;
        }

        struct MethodHolder<F, R, P>(pub F, PhantomData<(R, P)>);

        pub trait TypeConverter<T> {
            fn to_rust(context: &mut dyn $context, raw: $raw_type) -> T;
            fn from_rust(context: &mut dyn $context, rust: T) -> $raw_type;
        }

        pub trait MethodImpl<F, R, E, P> {
            fn into_body(self) -> Box<dyn MethodBody<E>>;
        }

        __generate!($context, $raw_type,);
        __generate!($context, $raw_type, P0);
        __generate!($context, $raw_type, P0, P1);
        __generate!($context, $raw_type, P0, P1, P2);
        __generate!($context, $raw_type, P0, P1, P2, P3);
        __generate!($context, $raw_type, P0, P1, P2, P3, P4);
        __generate!($context, $raw_type, P0, P1, P2, P3, P4, P5);
        __generate!($context, $raw_type, P0, P1, P2, P3, P4, P5, P6);
        __generate!($context, $raw_type, P0, P1, P2, P3, P4, P5, P6, P7);
        __generate!($context, $raw_type, P0, P1, P2, P3, P4, P5, P6, P7, P8);
    };
}

methods!(WIPICContext, WIPICWord);
