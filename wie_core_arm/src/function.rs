use alloc::{boxed::Box, format, string::String};
use core::{fmt::Debug, future::Future, marker::PhantomData};

use wie_util::read_null_terminated_string;

use crate::{ArmCore, ArmCoreError, ArmCoreResult};

#[async_trait::async_trait]
pub trait RegisteredFunction: Sync + Send {
    async fn call(&self, core: &mut ArmCore) -> ArmCoreResult<()>;
}

pub struct RegisteredFunctionHolder<F, C, R, E, P>
where
    F: EmulatedFunction<C, R, E, P> + 'static,
    E: Debug,
    C: Clone + 'static,
    R: ResultWriter<R>,
{
    function: Box<F>,
    context: C,
    _phantom: PhantomData<(C, R, E, P)>,
}

impl<F, C, R, E, P> RegisteredFunctionHolder<F, C, R, E, P>
where
    F: EmulatedFunction<C, R, E, P> + 'static,
    E: Debug,
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
impl<F, C, R, E, P> RegisteredFunction for RegisteredFunctionHolder<F, C, R, E, P>
where
    F: EmulatedFunction<C, R, E, P> + 'static + Sync + Send,
    E: Debug + Sync + Send,
    C: Clone + Sync + Send + 'static,
    R: ResultWriter<R> + Sync + Send,
    P: Sync + Send,
{
    async fn call(&self, core: &mut ArmCore) -> ArmCoreResult<()> {
        let (pc, lr) = core.read_pc_lr()?;

        tracing::trace!("Registered function called at {:#x}, LR: {:#x}", pc, lr);

        let mut new_context = self.context.clone();

        let result = self
            .function
            .call(core, &mut new_context)
            .await
            .map_err(|x| ArmCoreError::FunctionCallError(format!("{:?}", x)))?;
        R::write(core, result, lr)?;

        Ok(())
    }
}

trait FnHelper<'a, C, R, E, P> {
    type Output: Future<Output = Result<R, E>> + 'a + Send;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Self::Output;
}

macro_rules! generate_fn_helper {
    ($($arg: ident),*) => {
        impl<'a, F, Fut, C, R, E, $($arg),*> FnHelper<'a, C, R, E, ($($arg,)*)> for F
        where
            F: Fn(&'a mut ArmCore, &'a mut C, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a + Send,
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
pub trait EmulatedFunction<C, R, E, P> {
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E>;
}

macro_rules! generate_emulated_function {
    ($($arg: ident),*) => {
        #[async_trait::async_trait]
        impl<Func, C, R, E, $($arg),*> EmulatedFunction<C, R, E, ($($arg,)*)> for Func
        where
            Func: for<'a> FnHelper<'a, C, R, E, ($($arg,)*)> + Sync,
            C: Send,
            $($arg: EmulatedFunctionParam<$arg>),*
        {
            async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E> {
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
    fn write(core: &mut ArmCore, value: R, lr: u32) -> ArmCoreResult<()>;
}

impl ResultWriter<u32> for u32 {
    fn write(core: &mut ArmCore, value: u32, lr: u32) -> ArmCoreResult<()> {
        core.write_result(value, lr)
    }
}

impl ResultWriter<()> for () {
    fn write(core: &mut ArmCore, _: (), lr: u32) -> ArmCoreResult<()> {
        core.write_result(0, lr)
    }
}
