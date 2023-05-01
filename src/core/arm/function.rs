use unicorn_engine::RegisterARM;

use super::ArmCore;

pub trait EmulatedFunction<T, C> {
    fn call(&self, core: &mut ArmCore, context: C) -> u32;
}

impl<Func, C> EmulatedFunction<(), C> for Func
where
    Func: Fn(&mut ArmCore, C) -> u32,
{
    fn call(&self, core: &mut ArmCore, context: C) -> u32 {
        self(core, context)
    }
}

impl<Func, C, Param1> EmulatedFunction<(Param1,), C> for Func
where
    Func: Fn(&mut ArmCore, C, Param1) -> u32,
    Param1: EmulatedFunctionParam<Param1>,
{
    fn call(&self, core: &mut ArmCore, context: C) -> u32 {
        let param1 = Param1::get(core, 0);

        self(core, context, param1)
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

        core.read_null_terminated_string(ptr)
    }
}
