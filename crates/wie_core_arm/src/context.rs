use unicorn_engine::{RegisterARM, Unicorn};

use wie_base::CoreContext;

pub struct ArmCoreContext {
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub r3: u32,
    pub r4: u32,
    pub r5: u32,
    pub r6: u32,
    pub r7: u32,
    pub r8: u32,
    pub sb: u32,
    pub sl: u32,
    pub fp: u32,
    pub ip: u32,
    pub sp: u32,
    pub lr: u32,
    pub pc: u32,
}

impl ArmCoreContext {
    pub fn from_uc(uc: &Unicorn<'_, ()>) -> Self {
        Self {
            r0: uc.reg_read(RegisterARM::R0).unwrap() as u32,
            r1: uc.reg_read(RegisterARM::R1).unwrap() as u32,
            r2: uc.reg_read(RegisterARM::R2).unwrap() as u32,
            r3: uc.reg_read(RegisterARM::R3).unwrap() as u32,
            r4: uc.reg_read(RegisterARM::R4).unwrap() as u32,
            r5: uc.reg_read(RegisterARM::R5).unwrap() as u32,
            r6: uc.reg_read(RegisterARM::R6).unwrap() as u32,
            r7: uc.reg_read(RegisterARM::R7).unwrap() as u32,
            r8: uc.reg_read(RegisterARM::R8).unwrap() as u32,
            sb: uc.reg_read(RegisterARM::SB).unwrap() as u32,
            sl: uc.reg_read(RegisterARM::SL).unwrap() as u32,
            fp: uc.reg_read(RegisterARM::FP).unwrap() as u32,
            ip: uc.reg_read(RegisterARM::IP).unwrap() as u32,
            sp: uc.reg_read(RegisterARM::SP).unwrap() as u32,
            lr: uc.reg_read(RegisterARM::LR).unwrap() as u32,
            pc: uc.reg_read(RegisterARM::PC).unwrap() as u32,
        }
    }
}

impl CoreContext for ArmCoreContext {}
