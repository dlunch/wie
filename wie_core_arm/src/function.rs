use alloc::{boxed::Box, string::String};
use core::{future::Future, marker::PhantomData};

use wie_util::{read_null_terminated_string, Result};

use crate::ArmCore;

#[async_trait::async_trait]
pub trait RegisteredFunction: Sync + Send {
    async fn call(&self, core: &mut ArmCore) -> Result<()>;
}

pub struct RegisteredFunctionHolder<F, C, R, P>
where
    F: EmulatedFunction<C, R, P> + 'static,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    function: Box<F>,
    context: C,
    _phantom: PhantomData<(C, R, P)>,
}

impl<F, C, R, P> RegisteredFunctionHolder<F, C, R, P>
where
    F: EmulatedFunction<C, R, P> + 'static,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    pub fn new(function: F, context: &C) -> Self {
        Self {
            function: Box::new(function),
            context: context.clone(),
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<F, C, R, P> RegisteredFunction for RegisteredFunctionHolder<F, C, R, P>
where
    F: EmulatedFunction<C, R, P> + 'static + Sync + Send,
    C: Clone + Sync + Send + 'static,
    R: ResultWriter<R> + Sync + Send,
    P: Sync + Send,
{
    async fn call(&self, core: &mut ArmCore) -> Result<()> {
        let (pc, lr) = core.read_pc_lr()?;

        tracing::trace!("Registered function called at {:#x}, LR: {:#x}", pc, lr);

        let mut new_context = self.context.clone();

        let result = self.function.call(core, &mut new_context).await?;
        R::write(core, result, lr)?;

        Ok(())
    }
}

trait FnHelper<'a, C, R, P> {
    type Output: Future<Output = Result<R>> + 'a + Send;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Self::Output;
}

macro_rules! generate_fn_helper {
    ($($arg: ident),*) => {
        impl<'a, F, Fut, C, R, $($arg),*> FnHelper<'a, C, R, ($($arg,)*)> for F
        where
            F: Fn(&'a mut ArmCore, &'a mut C, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R>> + 'a + Send,
            C: 'a,
            R: 'a,
            $($arg: EmulatedFunctionParam<$arg>),*
        {
            type Output = Fut;
            #[allow(unused_variables, unused_assignments, non_snake_case, unused_mut)]
            fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Fut {
                let mut index = 0;
                $(
                    let $arg = $arg::get(core, index);
                    index += 1;
                )*
                self(core, context, $($arg),*)
            }
        }
    };
}

generate_fn_helper!();
generate_fn_helper!(P0);
generate_fn_helper!(P0, P1);
generate_fn_helper!(P0, P1, P2);
generate_fn_helper!(P0, P1, P2, P3);

#[async_trait::async_trait]
pub trait EmulatedFunction<C, R, P> {
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R>;
}

macro_rules! generate_emulated_function {
    ($($arg: ident),*) => {
        #[async_trait::async_trait]
        impl<Func, C, R, $($arg),*> EmulatedFunction<C, R, ($($arg,)*)> for Func
        where
            Func: for<'a> FnHelper<'a, C, R, ($($arg,)*)> + Sync,
            C: Send,
            $($arg: EmulatedFunctionParam<$arg>),*
        {
            async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R> {
                self.do_call(core, context).await
            }
        }
    };
}

generate_emulated_function!();
generate_emulated_function!(P0);
generate_emulated_function!(P0, P1);
generate_emulated_function!(P0, P1, P2);
generate_emulated_function!(P0, P1, P2, P3);

pub trait EmulatedFunctionParam<T> {
    fn get(core: &mut ArmCore, pos: usize) -> T;

    fn read(core: &mut ArmCore, pos: usize) -> u32 {
        core.read_param(pos).unwrap()
    }
}

impl EmulatedFunctionParam<String> for String {
    fn get(core: &mut ArmCore, pos: usize) -> String {
        let ptr = Self::read(core, pos);

        read_null_terminated_string(core, ptr).unwrap()
    }
}

impl EmulatedFunctionParam<u32> for u32 {
    fn get(core: &mut ArmCore, pos: usize) -> u32 {
        Self::read(core, pos)
    }
}

pub trait ResultWriter<R> {
    fn write(core: &mut ArmCore, value: R, lr: u32) -> Result<()>;
}

impl ResultWriter<u32> for u32 {
    fn write(core: &mut ArmCore, value: u32, lr: u32) -> Result<()> {
        core.write_result(value, lr)
    }
}

impl ResultWriter<()> for () {
    fn write(core: &mut ArmCore, _: (), lr: u32) -> Result<()> {
        core.write_result(0, lr)
    }
}
