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

const IMAGE_BASE: u32 = 0x100000;
const STACK_BASE: u32 = 0x70000000;
const STACK_SIZE: u32 = 0x10000;
const FUNCTIONS_BASE: u32 = 0x71000000;
const RUN_FUNCTION_LR: u32 = 0x7f000000;
const HEAP_BASE: u32 = 0x40000000;
static FUNCTIONS_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug)]
pub struct UnicornError(uc_error);

impl Error for UnicornError {}
impl Display for UnicornError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

pub struct ArmCore {
    uc: Unicorn<'static, ()>,
}

impl ArmCore {
    pub fn new() -> anyhow::Result<Self> {
        let mut uc = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).map_err(UnicornError)?;

        uc.add_block_hook(Self::block_hook).map_err(UnicornError)?;
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

    fn from_uc(uc: Unicorn<'static, ()>) -> Self {
        Self { uc }
    }

    pub fn load(&mut self, data: &[u8], map_size: usize) -> anyhow::Result<u32> {
        self.uc
            .mem_map(IMAGE_BASE as u64, round_up(map_size, 0x1000), Permission::ALL)
            .map_err(UnicornError)?;
        self.uc.mem_write(IMAGE_BASE as u64, data).map_err(UnicornError)?;

        Ok(IMAGE_BASE)
    }

    pub fn run_function(&mut self, address: u32, params: &[u32]) -> anyhow::Result<u32> {
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

    pub fn register_function<F, P, C, E>(&mut self, function: F, context: &C) -> anyhow::Result<u32>
    where
        F: for<'a> EmulatedFunction<P, &'a C, E> + 'static,
        E: std::fmt::Debug,
        C: Clone + 'static,
    {
        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE as u64 + FUNCTIONS_COUNT.fetch_add(2, Ordering::SeqCst) as u64;

        self.uc.mem_write(address, &bytes).map_err(UnicornError)?;

        let new_context = context.clone();
        self.uc
            .add_code_hook(address, address, move |uc, _, _| {
                log::trace!(
                    "Registered function called at {:#x}, LR: {:#x}",
                    address,
                    uc.reg_read(RegisterARM::LR).unwrap()
                );

                let mut new_self = Self::from_uc(Unicorn::try_from(uc.get_handle()).unwrap());

                let ret = function.call(&mut new_self, &new_context).unwrap();

                uc.reg_write(RegisterARM::R0, ret as u64).unwrap();
            })
            .map_err(UnicornError)?;

        log::trace!("Register function at {:#x}", address);

        Ok(address as u32 + 1)
    }

    pub fn alloc(&mut self, address: u32, size: u32) -> anyhow::Result<()> {
        log::trace!("Alloc address: {:#x}, size: {:#x}", address, size);

        self.uc
            .mem_map(address as u64, size as usize, Permission::READ | Permission::WRITE)
            .map_err(UnicornError)?;

        Ok(())
    }

    pub fn read<T>(&self, address: u32) -> anyhow::Result<T>
    where
        T: Copy,
    {
        let data = self.uc.mem_read_as_vec(address as u64, std::mem::size_of::<T>()).map_err(UnicornError)?;

        log::trace!("Read address: {:#x}, data: {:02x?}", address, data);

        Ok(unsafe { *(data.as_ptr() as *const T) })
    }

    pub fn read_null_terminated_string(&self, address: u32) -> anyhow::Result<String> {
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

        Ok(String::from_utf8(result)?)
    }

    pub fn write<T>(&mut self, address: u32, data: T) -> anyhow::Result<()> {
        let data_slice = unsafe { slice::from_raw_parts(&data as *const T as *const u8, std::mem::size_of::<T>()) };

        self.write_raw(address, data_slice)
    }

    pub fn write_raw(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()> {
        log::trace!("Write address: {:#x}, data: {:02x?}", address, data);

        self.uc.mem_write(address as u64, data).map_err(UnicornError)?;

        Ok(())
    }

    pub fn dump_regs(&self) -> anyhow::Result<String> {
        Self::dump_regs_inner(&self.uc)
    }

    fn dump_regs_inner(uc: &Unicorn<'_, ()>) -> anyhow::Result<String> {
        Ok([
            format!(
                "R0: {:#x} R1: {:#x} R2: {:#x} R3: {:#x} R4: {:#x} R5: {:#x} R6: {:#x} R7: {:#x} R8: {:#x}",
                uc.reg_read(RegisterARM::R0).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R1).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R2).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R3).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R4).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R5).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R6).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R7).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::R8).map_err(UnicornError)?,
            ),
            format!(
                "SB: {:#x} SL: {:#x} FP: {:#x} IP: {:#x} SP: {:#x} LR: {:#x} PC: {:#x}",
                uc.reg_read(RegisterARM::SB).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::SL).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::FP).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::IP).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::SP).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::LR).map_err(UnicornError)?,
                uc.reg_read(RegisterARM::PC).map_err(UnicornError)?,
            ),
            format!("APSR: {:032b}\n", uc.reg_read(RegisterARM::APSR).map_err(UnicornError)?),
        ]
        .join("\n"))
    }

    fn block_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        log::trace!("-- address: {:#x}, size: {:#x}", address, size);
        let insn = uc.mem_read_as_vec(address, size as usize).unwrap();

        let cs = Capstone::new()
            .arm()
            .mode(capstone::arch::arm::ArchMode::Thumb)
            .detail(true)
            .build()
            .unwrap();

        let insns = cs.disasm_all(&insn, address).unwrap();
        for insn in insns.iter() {
            log::trace!("{} {}", insn.mnemonic().unwrap(), insn.op_str().unwrap());
        }
        log::trace!("-- reg");

        log::trace!("\n{}", Self::dump_regs_inner(uc).unwrap());

        log::trace!("--");
    }

    fn mem_hook(uc: &mut Unicorn<'_, ()>, mem_type: MemType, address: u64, size: usize, value: i64) -> bool {
        let pc = uc.reg_read(RegisterARM::PC).unwrap();
        let lr = uc.reg_read(RegisterARM::LR).unwrap();

        if mem_type == MemType::READ {
            let value = uc.mem_read_as_vec(address, size).unwrap();
            if size == 4 {
                let value = u32::from_le_bytes(value.try_into().unwrap());
                log::trace!(
                    "pc: {:#x} lr: {:#x} mem_type: {:?} address: {:#x} size: {:#x} value: {:#x}",
                    pc,
                    lr,
                    mem_type,
                    address,
                    size,
                    value
                );
            } else {
                log::trace!(
                    "pc: {:#x} lr: {:#x} mem_type: {:?} address: {:#x} size: {:#x} value: {:?}",
                    pc,
                    lr,
                    mem_type,
                    address,
                    size,
                    value
                );
            }
        } else {
            log::error!("Invalid Memory Access");
            log::error!("mem_type: {:?} address: {:#x} size: {:#x} value: {:#x}", mem_type, address, size, value);
            log::error!("Register dump\n{}", Self::dump_regs_inner(uc).unwrap())
        }

        true
    }
}
