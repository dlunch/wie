use unicorn_engine::RegisterARM;

use super::ArmEmulator;

pub trait EmulatedFunction<T> {
    fn call(&self, emulator: &mut ArmEmulator) -> u32;
}

impl<Func> EmulatedFunction<()> for Func
where
    Func: Fn(&mut ArmEmulator) -> u32,
{
    fn call(&self, emulator: &mut ArmEmulator) -> u32 {
        self(emulator)
    }
}

impl<Func, Param1> EmulatedFunction<(Param1,)> for Func
where
    Func: Fn(&mut ArmEmulator, Param1) -> u32,
    Param1: EmulatedFunctionParam<Param1>,
{
    fn call(&self, emulator: &mut ArmEmulator) -> u32 {
        let param1 = Param1::get(emulator, 0);

        self(emulator, param1)
    }
}

pub trait EmulatedFunctionParam<T> {
    fn get(emulator: &mut ArmEmulator, pos: usize) -> T;

    fn read(emulator: &mut ArmEmulator, pos: usize) -> u32 {
        if pos == 0 {
            emulator.uc.reg_read(RegisterARM::R0).unwrap() as u32
        } else if pos == 1 {
            emulator.uc.reg_read(RegisterARM::R1).unwrap() as u32
        } else if pos == 2 {
            emulator.uc.reg_read(RegisterARM::R2).unwrap() as u32
        } else if pos == 3 {
            emulator.uc.reg_read(RegisterARM::R3).unwrap() as u32
        } else {
            todo!()
        }
    }
}

impl EmulatedFunctionParam<String> for String {
    fn get(emulator: &mut ArmEmulator, pos: usize) -> String {
        let ptr = Self::read(emulator, pos);

        emulator.read_null_terminated_string(ptr)
    }
}
