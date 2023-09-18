#[macro_export]
macro_rules! __impl_fn_helper {
    ($context: ident,) => {
        impl<'a, E, R, F, Fut> FnHelper<'a, E, R, ()> for F
        where
            F: Fn(&'a mut dyn $context) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a,
        {
            type Output = Fut;
            fn do_call(&self, context: &'a mut dyn $context, _args: &[u32]) -> Fut {
                self(context)
            }
        }
    };

    ($context: ident, $($arg: ident),*) => {
        impl<'a, E, R, F, Fut, $($arg),*> FnHelper<'a, E, R, ($($arg,)*)> for F
        where
            F: Fn(&'a mut dyn $context, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a,
            $($arg: TypeConverter<$arg> + 'a),*
        {
            type Output = Fut;
            #[allow(unused_assignments, non_snake_case)]
            fn do_call(&self, context: &'a mut dyn $context, args: &[u32]) -> Fut {
                let mut index = 0;
                $(
                    let $arg = $arg::to_rust(context, args[index]);
                    index += 1;
                )*
                self(context, $($arg),*)
            }
        }
    };
}

#[macro_export]
macro_rules! __impl_method_body {
    ($context: ident, $($arg: ident),*) => {
        #[async_trait::async_trait(?Send)]
        impl<F, R, E, $($arg),*> MethodBody<E> for MethodHolder<F, R, ($($arg,)*)>
        where
            F: for<'a> FnHelper<'a, E, R, ($($arg,)*)>,
            R: TypeConverter<R>,
        {
            async fn call(&self, context: &mut dyn $context, args: &[u32]) -> Result<u32, E> {
                let result = self.0.do_call(context, args).await?;

                Ok(R::from_rust(context, result))
            }
        }
    };
}

#[macro_export]
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

#[macro_export]
macro_rules! __generate {
    ($context: ident, $($arg: ident),*) => {
        $crate::__impl_fn_helper!($context, $($arg),*);
        $crate::__impl_method_body!($context, $($arg),*);
        $crate::__impl_method_impl!($($arg),*);
    };
}

#[macro_export]
macro_rules! methods {
    ($context: ident) => {
        #[async_trait::async_trait(?Send)]
        pub trait MethodBody<E> {
            async fn call(&self, context: &mut dyn $context, args: &[u32]) -> Result<u32, E>;
        }

        trait FnHelper<'a, E, R, P> {
            type Output: Future<Output = Result<R, E>> + 'a;
            fn do_call(&self, context: &'a mut dyn $context, args: &[u32]) -> Self::Output;
        }

        struct MethodHolder<F, R, P>(pub F, PhantomData<(R, P)>);

        pub trait TypeConverter<T> {
            fn to_rust(context: &mut dyn $context, raw: u32) -> T;
            fn from_rust(context: &mut dyn $context, rust: T) -> u32;
        }

        pub trait MethodImpl<F, R, E, P> {
            fn into_body(self) -> Box<dyn MethodBody<E>>;
        }

        $crate::__generate!($context,);
        $crate::__generate!($context, P0);
        $crate::__generate!($context, P0, P1);
        $crate::__generate!($context, P0, P1, P2);
        $crate::__generate!($context, P0, P1, P2, P3);
        $crate::__generate!($context, P0, P1, P2, P3, P4);
        $crate::__generate!($context, P0, P1, P2, P3, P4, P5);
        $crate::__generate!($context, P0, P1, P2, P3, P4, P5, P6);
        $crate::__generate!($context, P0, P1, P2, P3, P4, P5, P6, P7);
        $crate::__generate!($context, P0, P1, P2, P3, P4, P5, P6, P7, P8);
    };
}
