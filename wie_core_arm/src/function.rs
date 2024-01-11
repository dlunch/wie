use alloc::{boxed::Box, string::String};
use core::{fmt::Debug, future::Future, marker::PhantomData};

use wie_backend::SystemHandle;
use wie_common::util::read_null_terminated_string;

use crate::{engine::ArmEngineResult, ArmCore};

#[async_trait::async_trait(?Send)]
pub trait RegisteredFunction {
    async fn call(&self, core: &mut ArmCore, system: &mut SystemHandle) -> ArmEngineResult<()>;
}

pub struct RegisteredFunctionHolder<F, P, E, R>
where
    F: EmulatedFunction<P, E, R> + 'static,
    E: Debug,
    R: ResultWriter<R>,
{
    function: Box<F>,
    _phantom: PhantomData<(P, E, R)>,
}

impl<F, P, E, R> RegisteredFunctionHolder<F, P, E, R>
where
    F: EmulatedFunction<P, E, R> + 'static,
    E: Debug,
    R: ResultWriter<R>,
{
    pub fn new(function: F) -> Self {
        Self {
            function: Box::new(function),
            _phantom: PhantomData,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<F, P, E, R> RegisteredFunction for RegisteredFunctionHolder<F, P, E, R>
where
    F: EmulatedFunction<P, E, R> + 'static,
    E: Debug,
    R: ResultWriter<R>,
{
    async fn call(&self, core: &mut ArmCore, system: &mut SystemHandle) -> ArmEngineResult<()> {
        let (pc, lr) = core.read_pc_lr()?;

        tracing::trace!("Registered function called at {:#x}, LR: {:#x}", pc, lr);

        let result = self.function.call(core, system).await.map_err(|x| anyhow::anyhow!("{:?}", x))?;
        R::write(core, result, lr)?;

        Ok(())
    }
}

trait FnHelper<'a, E, R, P> {
    type Output: Future<Output = Result<R, E>> + 'a;
    fn do_call(&self, core: &'a mut ArmCore, system: &'a mut SystemHandle) -> Self::Output;
}

macro_rules! generate_fn_helper {
    ($($arg: ident),*) => {
        impl<'a, E, R, F, Fut, $($arg),*> FnHelper<'a, E, R, ($($arg,)*)> for F
        where
            F: Fn(&'a mut ArmCore, &'a mut SystemHandle, $($arg),*) -> Fut,
            Fut: Future<Output = Result<R, E>> + 'a,
            R: 'a,
            $($arg: EmulatedFunctionParam<$arg>),*
        {
            type Output = Fut;
            #[allow(unused_variables, unused_assignments, non_snake_case, unused_mut)]
            fn do_call(&self, core: &'a mut ArmCore, system: &'a mut SystemHandle) -> Fut {
                let mut index = 0;
                $(
                    let $arg = $arg::get(core, index);
                    index += 1;
                )*
                self(core, system, $($arg),*)
            }
        }
    };
}

generate_fn_helper!();
generate_fn_helper!(P0);
generate_fn_helper!(P0, P1);
generate_fn_helper!(P0, P1, P2);
generate_fn_helper!(P0, P1, P2, P3);

#[async_trait::async_trait(?Send)]
pub trait EmulatedFunction<P, E, R> {
    async fn call(&self, core: &mut ArmCore, system: &mut SystemHandle) -> Result<R, E>;
}

macro_rules! generate_emulated_function {
    ($($arg: ident),*) => {
        #[async_trait::async_trait(?Send)]
        impl<Func, E, R, $($arg),*> EmulatedFunction<($($arg,)*), E, R> for Func
        where
            Func: for<'a> FnHelper<'a, E, R, ($($arg,)*)>,
            $($arg: EmulatedFunctionParam<$arg>),*
        {
            async fn call(&self, core: &mut ArmCore, system: &mut SystemHandle) -> Result<R, E> {
                self.do_call(core, system).await
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
    fn write(core: &mut ArmCore, value: R, lr: u32) -> anyhow::Result<()>;
}

impl ResultWriter<u32> for u32 {
    fn write(core: &mut ArmCore, value: u32, lr: u32) -> anyhow::Result<()> {
        core.write_result(value, lr)
    }
}

impl ResultWriter<()> for () {
    fn write(core: &mut ArmCore, _: (), lr: u32) -> anyhow::Result<()> {
        core.write_result(0, lr)
    }
}
