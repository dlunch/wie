use unicorn_engine::RegisterARM;

use super::ArmCore;

pub trait EmulatedFunction<T> {
    fn call(&self, core: &mut ArmCore) -> u32;
}

impl<Func> EmulatedFunction<()> for Func
where
    Func: Fn(&mut ArmCore) -> u32,
{
    fn call(&self, core: &mut ArmCore) -> u32 {
        self(core)
    }
}

impl<Func, Param1> EmulatedFunction<(Param1,)> for Func
where
    Func: Fn(&mut ArmCore, Param1) -> u32,
    Param1: EmulatedFunctionParam<Param1>,
{
    fn call(&self, core: &mut ArmCore) -> u32 {
        let param1 = Param1::get(core, 0);

        self(core, param1)
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
