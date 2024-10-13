use alloc::boxed::Box;
use core::{future::Future, marker::PhantomData};

use crate::{WIPICContext, WIPICResult, WIPICWord};

macro_rules! __impl_fn_helper {
    ($context: ident, $raw_type: ty, $($arg: ident),*) => {
        impl<'a, E, R, F, Fut, $($arg),*> FnHelper<'a, E, R, ($($arg,)*)> for F
        where
            F: Fn(&'a mut dyn $context, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a + Send,
            $($arg: ParamConverter<$arg> + 'a),*
        {
            type Output = Fut;
            #[allow(unused_assignments, non_snake_case, unused_mut, unused_variables)]
            fn do_call(&self, context: &'a mut dyn $context, args: Box<[$raw_type]>) -> Fut {
                let mut args = alloc::vec::Vec::from(args).into_iter();
                $(
                    let $arg = $arg::convert(context, args.next().unwrap());
                )*
                self(context, $($arg),*)
            }
        }
    };
}

macro_rules! __impl_method_body {
    ($context: ident, $raw_type: ty, $($arg: ident),*) => {
        #[async_trait::async_trait]
        impl<F, R, E, $($arg),*> MethodBody<E> for MethodHolder<F, R, ($($arg,)*)>
        where
            F: for<'a> FnHelper<'a, E, R, ($($arg,)*)> + Sync + Send,
            R: ResultConverter<R> + Sync + Send,
            $($arg: Sync + Send),*
        {
            async fn call(&self, context: &mut dyn $context, args: Box<[$raw_type]>) -> Result<WIPICResult, E> {
                let result = self.0.do_call(context, args).await?;

                Ok(R::convert(context, result))
            }
        }
    };
}

macro_rules! __impl_method_impl {
    ($($arg: ident),*) => {
        impl<F, R, E, $($arg),*> MethodImpl<F, R, E, ($($arg,)*)> for F
        where
            F: for<'a> FnHelper<'a, E, R, ($($arg,)*)> + 'static + Sync + Send,
            R: ResultConverter<R> + 'static + Sync + Send,
            $($arg: 'static + Sync + Send),*
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
        #[async_trait::async_trait]
        pub trait MethodBody<E>: Sync + Send {
            async fn call(&self, context: &mut dyn $context, args: Box<[$raw_type]>) -> Result<WIPICResult, E>;
        }

        trait FnHelper<'a, E, R, P> {
            type Output: Future<Output = Result<R, E>> + 'a + Send;
            fn do_call(&self, context: &'a mut dyn $context, args: Box<[$raw_type]>) -> Self::Output;
        }

        struct MethodHolder<F, R, P>(pub F, PhantomData<(R, P)>);

        pub trait ParamConverter<T> {
            fn convert(context: &mut dyn $context, raw: $raw_type) -> T;
        }

        pub trait ResultConverter<T> {
            fn convert(context: &mut dyn $context, result: T) -> WIPICResult;
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
