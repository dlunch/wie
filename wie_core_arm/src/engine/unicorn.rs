use alloc::{format, vec::Vec};
use core::ops::Range;

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{uc_error, Arch, HookType, MemType, Mode, Permission},
    RegisterARM, Unicorn,
};

use crate::{
    engine::{ArmEngine, ArmEngineResult, ArmRegister, MemoryPermission},
    ArmCore,
};

pub struct UnicornEngine {
    uc: Unicorn<'static, ()>,
}

impl UnicornEngine {
    pub fn new() -> Self {
        let mut uc = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).unwrap();

        // uc.add_block_hook(Self::code_hook).map_err(UnicornError)?;
        // uc.add_code_hook(0, 0xffff_ffff_ffff_ffff, Self::code_hook).unwrap();
        uc.add_mem_hook(HookType::MEM_INVALID, 0, 0xffff_ffff_ffff_ffff, Self::mem_hook).unwrap();

        Self { uc }
    }

    #[allow(dead_code)]
    #[allow(unknown_lints)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn code_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        let insn = uc.mem_read_as_vec(address, size as usize).unwrap();
        let cpsr = uc.reg_read(RegisterARM::CPSR).unwrap();
        let thumb = cpsr & (1 << 5) != 0;

        let mut cs_builder = Capstone::new().arm();
        if thumb {
            cs_builder = cs_builder.mode(capstone::arch::arm::ArchMode::Thumb);
        } else {
            cs_builder = cs_builder.mode(capstone::arch::arm::ArchMode::Arm);
        }
        let cs = cs_builder.detail(true).build().unwrap();

        let insns = cs.disasm_all(&insn, address).unwrap();

        let insn_str = insns
            .iter()
            .map(|x| format!("{:#x}: {} {}", x.address(), x.mnemonic().unwrap(), x.op_str().unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        let engine = UnicornEngine {
            uc: Unicorn::try_from(uc.get_handle()).unwrap(),
        };
        tracing::trace!("{}\n{}", insn_str, ArmCore::dump_regs_inner(&engine));
    }

    #[allow(unknown_lints)]
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn mem_hook(uc: &mut Unicorn<'_, ()>, mem_type: MemType, address: u64, size: usize, value: i64) -> bool {
        let pc = uc.reg_read(RegisterARM::PC).unwrap();
        let lr = uc.reg_read(RegisterARM::LR).unwrap();

        if mem_type == MemType::READ || mem_type == MemType::FETCH || mem_type == MemType::WRITE {
            let value_str = if mem_type == MemType::WRITE {
                format!("{:#x}", value)
            } else {
                let value = uc.mem_read_as_vec(address, size).unwrap();

                if size == 4 {
                    format!("{:#x}", u32::from_le_bytes(value.try_into().unwrap()))
                } else {
                    format!("{:?}", value)
                }
            };

            tracing::trace!(
                "pc: {:#x} lr: {:#x} mem_type: {:?} address: {:#x} size: {:#x} value: {}",
                pc,
                lr,
                mem_type,
                address,
                size,
                value_str
            );

            true
        } else {
            let engine = UnicornEngine {
                uc: Unicorn::try_from(uc.get_handle()).unwrap(),
            };

            tracing::error!(
                "Invalid Memory Access\n\
                mem_type: {:?} address: {:#x} size: {:#x} value: {:#x}\n{}",
                mem_type,
                address,
                size,
                value,
                ArmCore::dump_regs_inner(&engine)
            );

            false
        }
    }
}

impl ArmEngine for UnicornEngine {
    fn run(&mut self, end: u32, hook: Range<u32>, count: u32) -> ArmEngineResult<()> {
        let hook = self
            .uc
            .add_code_hook(hook.start as u64, hook.end as u64, |uc, _, _| uc.emu_stop().unwrap())
            .unwrap();

        let cpsr = self.uc.reg_read(RegisterARM::CPSR).unwrap();
        let thumb = cpsr & (1 << 5) != 0;

        let pc = if thumb {
            self.uc.reg_read(RegisterARM::PC).unwrap() + 1
        } else {
            self.uc.reg_read(RegisterARM::PC).unwrap()
        };

        self.uc.emu_start(pc, end as u64, 0, count as _).map_err(UnicornError)?;
        self.uc.remove_hook(hook).unwrap();

        Ok(())
    }

    fn reg_write(&mut self, reg: ArmRegister, value: u32) {
        self.uc.reg_write(reg.into_unicorn(), value as u64).unwrap();
    }

    fn reg_read(&self, reg: ArmRegister) -> u32 {
        self.uc.reg_read(reg.into_unicorn()).unwrap() as u32
    }

    fn mem_map(&mut self, address: u32, size: usize, permission: MemoryPermission) {
        self.uc.mem_map(address as u64, size, permission.into_unicorn()).unwrap();
    }

    fn mem_write(&mut self, address: u32, data: &[u8]) -> ArmEngineResult<()> {
        self.uc.mem_write(address as u64, data).map_err(UnicornError)?;

        Ok(())
    }

    fn mem_read(&mut self, address: u32, size: usize) -> ArmEngineResult<Vec<u8>> {
        Ok(self.uc.mem_read_as_vec(address as u64, size).map_err(UnicornError)?)
    }
}

impl ArmRegister {
    fn into_unicorn(self) -> RegisterARM {
        match self {
            Self::R0 => RegisterARM::R0,
            Self::R1 => RegisterARM::R1,
            Self::R2 => RegisterARM::R2,
            Self::R3 => RegisterARM::R3,
            Self::R4 => RegisterARM::R4,
            Self::R5 => RegisterARM::R5,
            Self::R6 => RegisterARM::R6,
            Self::R7 => RegisterARM::R7,
            Self::R8 => RegisterARM::R8,
            Self::SB => RegisterARM::SB,
            Self::SL => RegisterARM::SL,
            Self::FP => RegisterARM::FP,
            Self::IP => RegisterARM::IP,
            Self::SP => RegisterARM::SP,
            Self::LR => RegisterARM::LR,
            Self::PC => RegisterARM::PC,
            Self::Cpsr => RegisterARM::CPSR,
        }
    }
}

impl MemoryPermission {
    fn into_unicorn(self) -> Permission {
        match self {
            Self::ReadExecute => Permission::READ | Permission::EXEC,
            Self::ReadWrite => Permission::READ | Permission::WRITE,
            Self::ReadWriteExecute => Permission::READ | Permission::WRITE | Permission::EXEC,
        }
    }
}

pub struct UnicornError(uc_error);

impl From<UnicornError> for anyhow::Error {
    fn from(err: UnicornError) -> Self {
        anyhow::anyhow!("{:?}", err.0)
    }
}
