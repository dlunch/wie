use alloc::{boxed::Box, string::String};
use core::future::Future;

use unicorn_engine::RegisterARM;

use wie_base::util::read_null_terminated_string;

use crate::ArmCore;

trait FnHelper<'a, E, C, R, P> {
    type Output: Future<Output = Result<R, E>> + 'a;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Self::Output;
}

impl<'a, E, C, R, F, Fut> FnHelper<'a, E, C, R, ()> for F
where
    F: Fn(&'a mut ArmCore, &'a mut C) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    C: 'a,
    R: 'a,
{
    type Output = Fut;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Fut {
        self(core, context)
    }
}

impl<'a, E, C, R, F, Fut, Param1> FnHelper<'a, E, C, R, (Param1,)> for F
where
    F: Fn(&'a mut ArmCore, &'a mut C, Param1) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    C: 'a,
    R: 'a,
    Param1: EmulatedFunctionParam<Param1>,
{
    type Output = Fut;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Fut {
        let param1 = Param1::get(core, 0);

        self(core, context, param1)
    }
}

impl<'a, E, C, R, F, Fut, Param1, Param2> FnHelper<'a, E, C, R, (Param1, Param2)> for F
where
    F: Fn(&'a mut ArmCore, &'a mut C, Param1, Param2) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    C: 'a,
    R: 'a,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
{
    type Output = Fut;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Fut {
        let param1 = Param1::get(core, 0);
        let param2 = Param2::get(core, 1);

        self(core, context, param1, param2)
    }
}

impl<'a, E, C, R, F, Fut, Param1, Param2, Param3> FnHelper<'a, E, C, R, (Param1, Param2, Param3)> for F
where
    F: Fn(&'a mut ArmCore, &'a mut C, Param1, Param2, Param3) -> Fut,
    Fut: Future<Output = Result<R, E>> + 'a,
    C: 'a,
    R: 'a,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
    Param3: EmulatedFunctionParam<Param3>,
{
    type Output = Fut;
    fn do_call(&self, core: &'a mut ArmCore, context: &'a mut C) -> Fut {
        let param1 = Param1::get(core, 0);
        let param2 = Param2::get(core, 1);
        let param3 = Param3::get(core, 2);

        self(core, context, param1, param2, param3)
    }
}

#[async_trait::async_trait(?Send)]
pub trait EmulatedFunction<T, E, C, R> {
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E>;
}

#[async_trait::async_trait(?Send)]
impl<Func, E, C, R> EmulatedFunction<(), E, C, R> for Func
where
    Func: for<'a> FnHelper<'a, E, C, R, ()>,
{
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E> {
        self.do_call(core, context).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Func, E, C, R, Param1> EmulatedFunction<(Param1,), E, C, R> for Func
where
    Func: for<'a> FnHelper<'a, E, C, R, (Param1,)>,
    Param1: EmulatedFunctionParam<Param1>,
{
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E> {
        self.do_call(core, context).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Func, E, C, R, Param1, Param2> EmulatedFunction<(Param1, Param2), E, C, R> for Func
where
    Func: for<'a> FnHelper<'a, E, C, R, (Param1, Param2)>,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
{
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E> {
        self.do_call(core, context).await
    }
}

#[async_trait::async_trait(?Send)]
impl<Func, E, C, R, Param1, Param2, Param3> EmulatedFunction<(Param1, Param2, Param3), E, C, R> for Func
where
    Func: for<'a> FnHelper<'a, E, C, R, (Param1, Param2, Param3)>,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
    Param3: EmulatedFunctionParam<Param3>,
{
    async fn call(&self, core: &mut ArmCore, context: &mut C) -> Result<R, E> {
        self.do_call(core, context).await
    }
}

pub trait EmulatedFunctionParam<T> {
    fn get(core: &mut ArmCore, pos: usize) -> T;

    fn read(core: &mut ArmCore, pos: usize) -> u32 {
        if pos == 0 {
            core.uc.reg_read(RegisterARM::R0).unwrap() as u32
        } else if pos == 1 {
            core.uc.reg_read(RegisterARM::R1).unwrap() as u32
        } else if pos == 2 {
            core.uc.reg_read(RegisterARM::R2).unwrap() as u32
        } else if pos == 3 {
            core.uc.reg_read(RegisterARM::R3).unwrap() as u32
        } else {
            todo!()
        }
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
