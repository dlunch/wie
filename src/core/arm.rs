pub mod allocator;
mod function;

use std::{
    error::Error,
    fmt::{self, Display, Formatter},
    slice,
    sync::atomic::{AtomicU32, Ordering},
};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{uc_error, Arch, HookType, MemType, Mode, Permission},
    RegisterARM, Unicorn,
};

use crate::util::round_up;

use self::function::EmulatedFunction;

pub use function::EmulatedFunctionParam;

const IMAGE_BASE: u32 = 0x100000;
const STACK_BASE: u32 = 0x70000000;
const STACK_SIZE: u32 = 0x10000;
const FUNCTIONS_BASE: u32 = 0x71000000;
const RUN_FUNCTION_LR: u32 = 0x7f000000;
const HEAP_BASE: u32 = 0x40000000;
pub const PEB_BASE: u32 = 0x7ff00000;
static FUNCTIONS_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct UnicornError(uc_error);

impl Error for UnicornError {}
impl Display for UnicornError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub type ArmCoreResult<T> = anyhow::Result<T>;

pub struct ArmCore {
    uc: Unicorn<'static, ()>,
}

impl ArmCore {
    pub fn new() -> ArmCoreResult<Self> {
        let mut uc = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).map_err(UnicornError)?;

        uc.add_code_hook(0, 0xffffffff, Self::code_hook).map_err(UnicornError)?;
        uc.add_mem_hook(HookType::MEM_INVALID, 0, 0xffff_ffff_ffff_ffff, Self::mem_hook)
            .map_err(UnicornError)?;

        uc.mem_map(STACK_BASE as u64, STACK_SIZE as usize, Permission::READ | Permission::WRITE)
            .map_err(UnicornError)?;
        uc.mem_map(FUNCTIONS_BASE as u64, 0x1000, Permission::READ | Permission::EXEC)
            .map_err(UnicornError)?;

        uc.reg_write(RegisterARM::CPSR, 0x40000010).map_err(UnicornError)?; // usr32
        uc.reg_write(RegisterARM::SP, STACK_BASE as u64 + STACK_SIZE as u64)
            .map_err(UnicornError)?;

        Ok(Self { uc })
    }

    pub fn load(&mut self, data: &[u8], map_size: usize) -> ArmCoreResult<u32> {
        self.uc
            .mem_map(IMAGE_BASE as u64, round_up(map_size, 0x1000), Permission::ALL)
            .map_err(UnicornError)?;
        self.uc.mem_write(IMAGE_BASE as u64, data).map_err(UnicornError)?;

        Ok(IMAGE_BASE)
    }

    pub fn run_function(&mut self, address: u32, params: &[u32]) -> ArmCoreResult<u32> {
        // is there cleaner way to do this?
        if !params.is_empty() {
            self.uc.reg_write(RegisterARM::R0, params[0] as u64).map_err(UnicornError)?;
        }
        if params.len() > 1 {
            self.uc.reg_write(RegisterARM::R1, params[1] as u64).map_err(UnicornError)?;
        }
        if params.len() > 2 {
            self.uc.reg_write(RegisterARM::R2, params[2] as u64).map_err(UnicornError)?;
        }
        if params.len() > 3 {
            self.uc.reg_write(RegisterARM::R3, params[3] as u64).map_err(UnicornError)?;
        }
        if params.len() > 4 {
            for param in params[4..].iter() {
                let sp = self.uc.reg_read(RegisterARM::SP).map_err(UnicornError)?;

                self.uc.reg_write(RegisterARM::SP, sp - 4).map_err(UnicornError)?;
                self.uc.mem_write(sp - 4, &param.to_le_bytes()).map_err(UnicornError)?;
            }
        }

        log::trace!("Run function start {:#x}, params {:?}", address, params);

        let previous_lr = self.uc.reg_read(RegisterARM::LR).map_err(UnicornError)?; // TODO do we have to save more callee-saved registers?

        self.uc.reg_write(RegisterARM::LR, RUN_FUNCTION_LR as u64).map_err(UnicornError)?;
        self.uc.emu_start(address as u64, RUN_FUNCTION_LR as u64, 0, 0).map_err(UnicornError)?;

        let result = self.uc.reg_read(RegisterARM::R0).map_err(UnicornError)? as u32;

        log::trace!("Run function end, result: {:#x}", result);

        self.uc.reg_write(RegisterARM::LR, previous_lr).map_err(UnicornError)?;

        Ok(result)
    }

    pub fn register_function<F, P, E>(&mut self, function: F) -> ArmCoreResult<u32>
    where
        F: EmulatedFunction<P, E> + 'static,
        E: std::fmt::Debug,
    {
        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE as u64 + FUNCTIONS_COUNT.fetch_add(2, Ordering::SeqCst) as u64;

        self.uc.mem_write(address, &bytes).map_err(UnicornError)?;

        self.uc
            .add_code_hook(address, address, move |uc, _, _| {
                log::trace!(
                    "Registered function called at {:#x}, LR: {:#x}",
                    address,
                    uc.reg_read(RegisterARM::LR).unwrap()
                );

                let new_self = Self {
                    uc: Unicorn::try_from(uc.get_handle()).unwrap(),
                };
                let ret = function.call(new_self).unwrap();

                uc.reg_write(RegisterARM::R0, ret as u64).unwrap();
            })
            .map_err(UnicornError)?;

        log::trace!("Register function at {:#x}", address);

        Ok(address as u32 + 1)
    }

    pub fn alloc(&mut self, address: u32, size: u32) -> ArmCoreResult<()> {
        log::trace!("Alloc address: {:#x}, size: {:#x}", address, size);

        self.uc
            .mem_map(address as u64, size as usize, Permission::READ | Permission::WRITE)
            .map_err(UnicornError)?;

        Ok(())
    }

    pub fn read<T>(&self, address: u32) -> ArmCoreResult<T>
    where
        T: Copy,
    {
        let data = self.uc.mem_read_as_vec(address as u64, std::mem::size_of::<T>()).map_err(UnicornError)?;

        log::trace!("Read address: {:#x}, data: {:02x?}", address, data);

        Ok(unsafe { *(data.as_ptr() as *const T) })
    }

    pub fn read_null_terminated_string(&self, address: u32) -> ArmCoreResult<String> {
        // TODO we can read by 4bytes at once

        let mut result = Vec::new();
        let mut cursor = address;
        loop {
            let mut item = [0];
            self.uc.mem_read(cursor as u64, &mut item).map_err(UnicornError)?;
            cursor += 1;

            if item[0] == 0 {
                break;
            }

            result.push(item[0]);
        }

        log::trace!("Read address: {:#x}, data: {:02x?}", address, result);

        Ok(String::from_utf8(result)?)
    }

    pub fn write<T>(&mut self, address: u32, data: T) -> ArmCoreResult<()> {
        let data_slice = unsafe { slice::from_raw_parts(&data as *const T as *const u8, std::mem::size_of::<T>()) };

        self.write_raw(address, data_slice)
    }

    pub fn write_raw(&mut self, address: u32, data: &[u8]) -> ArmCoreResult<()> {
        log::trace!("Write address: {:#x}, data: {:02x?}", address, data);

        self.uc.mem_write(address as u64, data).map_err(UnicornError)?;

        Ok(())
    }

    pub fn dump_regs(&self) -> ArmCoreResult<String> {
        Self::dump_regs_inner(&self.uc)
    }

    fn dump_regs_inner(uc: &Unicorn<'_, ()>) -> ArmCoreResult<String> {
        let value = (|| {
            Ok::<_, uc_error>(
                [
                    format!(
                        "R0: {:#x} R1: {:#x} R2: {:#x} R3: {:#x} R4: {:#x} R5: {:#x} R6: {:#x} R7: {:#x} R8: {:#x}",
                        uc.reg_read(RegisterARM::R0)?,
                        uc.reg_read(RegisterARM::R1)?,
                        uc.reg_read(RegisterARM::R2)?,
                        uc.reg_read(RegisterARM::R3)?,
                        uc.reg_read(RegisterARM::R4)?,
                        uc.reg_read(RegisterARM::R5)?,
                        uc.reg_read(RegisterARM::R6)?,
                        uc.reg_read(RegisterARM::R7)?,
                        uc.reg_read(RegisterARM::R8)?,
                    ),
                    format!(
                        "SB: {:#x} SL: {:#x} FP: {:#x} IP: {:#x} SP: {:#x} LR: {:#x} PC: {:#x}",
                        uc.reg_read(RegisterARM::SB)?,
                        uc.reg_read(RegisterARM::SL)?,
                        uc.reg_read(RegisterARM::FP)?,
                        uc.reg_read(RegisterARM::IP)?,
                        uc.reg_read(RegisterARM::SP)?,
                        uc.reg_read(RegisterARM::LR)?,
                        uc.reg_read(RegisterARM::PC)?,
                    ),
                    format!("APSR: {:032b}\n", uc.reg_read(RegisterARM::APSR)?),
                ]
                .join("\n"),
            )
        })()
        .map_err(UnicornError)?;

        Ok(value)
    }

    fn code_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        let insn = uc.mem_read_as_vec(address, size as usize).unwrap();

        let cs = Capstone::new()
            .arm()
            .mode(capstone::arch::arm::ArchMode::Thumb)
            .detail(true)
            .build()
            .unwrap();

        let insns = cs.disasm_all(&insn, address).unwrap();

        let insn_str = insns
            .iter()
            .map(|x| format!("{:#x}: {} {}", x.address(), x.mnemonic().unwrap(), x.op_str().unwrap()))
            .collect::<Vec<_>>()
            .join("\n");

        log::trace!("{}\n{}", insn_str, Self::dump_regs_inner(uc).unwrap());
    }

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

            log::trace!(
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
            log::error!(
                "Invalid Memory Access\n\
                mem_type: {:?} address: {:#x} size: {:#x} value: {:#x}\n{}",
                mem_type,
                address,
                size,
                value,
                Self::dump_regs_inner(uc).unwrap()
            );

            false
        }
    }
}
