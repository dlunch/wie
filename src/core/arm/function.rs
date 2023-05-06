use unicorn_engine::RegisterARM;

use super::ArmCore;

pub trait EmulatedFunction<T, E, C> {
    fn call(&self, core: ArmCore, context: C) -> Result<u32, E>;
}

impl<Func, E, C> EmulatedFunction<(), E, C> for Func
where
    Func: Fn(ArmCore, C) -> Result<u32, E>,
{
    fn call(&self, core: ArmCore, context: C) -> Result<u32, E> {
        self(core, context)
    }
}

impl<Func, E, C, Param1> EmulatedFunction<(Param1,), E, C> for Func
where
    Func: Fn(ArmCore, C, Param1) -> Result<u32, E>,
    Param1: EmulatedFunctionParam<Param1>,
{
    fn call(&self, mut core: ArmCore, context: C) -> Result<u32, E> {
        let param1 = Param1::get(&mut core, 0);

        self(core, context, param1)
    }
}

impl<Func, E, C, Param1, Param2> EmulatedFunction<(Param1, Param2), E, C> for Func
where
    Func: Fn(ArmCore, C, Param1, Param2) -> Result<u32, E>,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
{
    fn call(&self, mut core: ArmCore, context: C) -> Result<u32, E> {
        let param1 = Param1::get(&mut core, 0);
        let param2 = Param2::get(&mut core, 1);

        self(core, context, param1, param2)
    }
}

impl<Func, E, C, Param1, Param2, Param3> EmulatedFunction<(Param1, Param2, Param3), E, C> for Func
where
    Func: Fn(ArmCore, C, Param1, Param2, Param3) -> Result<u32, E>,
    Param1: EmulatedFunctionParam<Param1>,
    Param2: EmulatedFunctionParam<Param2>,
    Param3: EmulatedFunctionParam<Param3>,
{
    fn call(&self, mut core: ArmCore, context: C) -> Result<u32, E> {
        let param1 = Param1::get(&mut core, 0);
        let param2 = Param2::get(&mut core, 1);
        let param3 = Param3::get(&mut core, 2);

        self(core, context, param1, param2, param3)
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

        core.read_null_terminated_string(ptr).unwrap()
    }
}

impl EmulatedFunctionParam<u32> for u32 {
    fn get(core: &mut ArmCore, pos: usize) -> u32 {
        Self::read(core, pos)
    }
}
