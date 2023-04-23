mod function;

use std::{
    slice,
    sync::atomic::{AtomicU32, Ordering},
};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{Arch, HookType, MemType, Mode, Permission},
    RegisterARM, Unicorn,
};

use crate::util::round_up;

use function::EmulatedFunction;

const IMAGE_BASE: u64 = 0x100000;
const STACK_BASE: u64 = 0x70000000;
const STACK_SIZE: usize = 0x10000;
const FUNCTIONS_BASE: u64 = 0x71000000;
const RUN_FUNCTION_LR: u64 = 0x7f000000;
static FUNCTIONS_COUNT: AtomicU32 = AtomicU32::new(0);

pub struct ArmEmulator {
    uc: Unicorn<'static, ()>,
}

impl ArmEmulator {
    pub fn new() -> Self {
        let mut uc = Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN).unwrap();

        uc.add_block_hook(Self::block_hook).unwrap();
        uc.add_mem_hook(HookType::MEM_FETCH_UNMAPPED, 0, 0xffff_ffff_ffff_ffff, Self::mem_hook)
            .unwrap();

        uc.mem_map(STACK_BASE, STACK_SIZE, Permission::READ | Permission::WRITE).unwrap();
        uc.mem_map(FUNCTIONS_BASE, 0x1000, Permission::READ | Permission::EXEC).unwrap();

        uc.reg_write(RegisterARM::CPSR, 0x40000010).unwrap(); // usr32
        uc.reg_write(RegisterARM::SP, STACK_BASE + STACK_SIZE as u64).unwrap();

        Self { uc }
    }

    fn from_uc(uc: Unicorn<'static, ()>) -> Self {
        Self { uc }
    }

    pub fn load(&mut self, data: &[u8], map_size: usize) -> u32 {
        self.uc.mem_map(IMAGE_BASE, round_up(map_size, 0x1000), Permission::ALL).unwrap();
        self.uc.mem_write(IMAGE_BASE, data).unwrap();

        IMAGE_BASE as u32
    }

    pub fn run_function(&mut self, address: u32, params: &[u32]) -> u32 {
        // is there cleaner way to do this?
        #[allow(clippy::len_zero)]
        if params.len() > 0 {
            self.uc.reg_write(RegisterARM::R0, params[0] as u64).unwrap();
        }
        if params.len() > 1 {
            self.uc.reg_write(RegisterARM::R1, params[1] as u64).unwrap();
        }
        if params.len() > 2 {
            self.uc.reg_write(RegisterARM::R2, params[2] as u64).unwrap();
        }
        if params.len() > 3 {
            self.uc.reg_write(RegisterARM::R3, params[3] as u64).unwrap();
        }
        if params.len() > 4 {
            for param in params[4..].iter() {
                self.uc
                    .reg_write(RegisterARM::SP, self.uc.reg_read(RegisterARM::SP).unwrap() - 4)
                    .unwrap();
                self.uc
                    .mem_write(self.uc.reg_read(RegisterARM::SP).unwrap(), &param.to_le_bytes())
                    .unwrap();
            }
        }

        log::debug!("Run function start {:#x}, params {:?}", address, params);

        self.uc.reg_write(RegisterARM::LR, RUN_FUNCTION_LR).unwrap();
        self.uc.emu_start(address as u64, RUN_FUNCTION_LR, 0, 0).unwrap();

        log::debug!("Run function end");

        self.uc.reg_read(RegisterARM::R0).unwrap() as u32
    }

    pub fn register_function<F, P>(&mut self, function: F) -> u32
    where
        F: EmulatedFunction<P> + 'static,
    {
        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE + FUNCTIONS_COUNT.fetch_add(2, Ordering::SeqCst) as u64;

        self.uc.mem_write(address, &bytes).unwrap();

        self.uc
            .add_code_hook(address, address + 2, move |uc, _, _| {
                log::debug!(
                    "Registered function called at {:#x}, LR: {:#x}",
                    address,
                    uc.reg_read(RegisterARM::LR).unwrap()
                );

                let mut new_self = Self::from_uc(Unicorn::try_from(uc.get_handle()).unwrap());

                let ret = function.call(&mut new_self);

                uc.reg_write(RegisterARM::R0, ret as u64).unwrap();
            })
            .unwrap();

        log::debug!("Register function at {:#x}", address);

        address as u32 + 1
    }

    pub fn alloc(&mut self, address: u32, size: u32) {
        log::debug!("Alloc address: {:#x}, size: {:#x}", address, size);

        self.uc
            .mem_map(address as u64, size as usize, Permission::READ | Permission::WRITE)
            .unwrap();
    }

    pub fn read<T>(&self, address: u32) -> T
    where
        T: Copy,
    {
        let data = self.uc.mem_read_as_vec(address as u64, std::mem::size_of::<T>()).unwrap();

        log::debug!("Read address: {:#x}, data: {:02x?}", address, data);

        unsafe { *(data.as_ptr() as *const T) }
    }

    pub fn read_null_terminated_string(&self, address: u32) -> String {
        // TODO we can read by 4bytes at once

        let mut result = Vec::new();
        let mut cursor = address;
        loop {
            let mut item = [0];
            self.uc.mem_read(cursor as u64, &mut item).unwrap();
            cursor += 1;

            if item[0] == 0 {
                break;
            }

            result.push(item[0]);
        }

        String::from_utf8(result).unwrap()
    }

    pub fn write<T>(&mut self, address: u32, data: T) {
        let data_slice = unsafe { slice::from_raw_parts(&data as *const T as *const u8, std::mem::size_of::<T>()) };

        log::debug!("Write address: {:#x}, data: {:02x?}", address, data_slice);

        self.uc.mem_write(address as u64, data_slice).unwrap();
    }

    pub fn free(&mut self, address: u32, size: u32) {
        log::debug!("Free address: {:#x}, size: {:#x}", address, size);

        self.uc.mem_unmap(address as u64, size as usize).unwrap()
    }

    pub fn dump_regs(&self) -> String {
        Self::dump_regs_inner(&self.uc)
    }

    fn dump_regs_inner(uc: &Unicorn<'_, ()>) -> String {
        [
            format!(
                "R0: {:x} R1: {:x} R2: {:x} R3: {:x} R4: {:x} R5: {:x} R6: {:x} R7: {:x} R8: {:x}",
                uc.reg_read(RegisterARM::R0).unwrap(),
                uc.reg_read(RegisterARM::R1).unwrap(),
                uc.reg_read(RegisterARM::R2).unwrap(),
                uc.reg_read(RegisterARM::R3).unwrap(),
                uc.reg_read(RegisterARM::R4).unwrap(),
                uc.reg_read(RegisterARM::R5).unwrap(),
                uc.reg_read(RegisterARM::R6).unwrap(),
                uc.reg_read(RegisterARM::R7).unwrap(),
                uc.reg_read(RegisterARM::R8).unwrap()
            ),
            format!(
                "SB: {:x} SL: {:x} FP: {:x} IP: {:x} SP: {:x} LR: {:x} PC: {:x}",
                uc.reg_read(RegisterARM::SB).unwrap(),
                uc.reg_read(RegisterARM::SL).unwrap(),
                uc.reg_read(RegisterARM::FP).unwrap(),
                uc.reg_read(RegisterARM::IP).unwrap(),
                uc.reg_read(RegisterARM::SP).unwrap(),
                uc.reg_read(RegisterARM::LR).unwrap(),
                uc.reg_read(RegisterARM::PC).unwrap()
            ),
            format!("APSR: {:032b}\n", uc.reg_read(RegisterARM::APSR).unwrap()),
        ]
        .join("\n")
    }

    fn block_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        log::trace!("-- address: {:x}, size: {:x}", address, size);
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

        log::trace!("\n{}", Self::dump_regs_inner(uc));

        log::trace!("--");
    }

    fn mem_hook(uc: &mut Unicorn<'_, ()>, mem_type: MemType, address: u64, size: usize, value: i64) -> bool {
        let pc = uc.reg_read(RegisterARM::PC).unwrap();

        if mem_type == MemType::READ {
            let value = uc.mem_read_as_vec(address, size).unwrap();
            if size == 4 {
                let value = u32::from_le_bytes(value.try_into().unwrap());
                log::trace!(
                    "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:x}",
                    pc,
                    mem_type,
                    address,
                    size,
                    value
                );
            } else {
                log::trace!(
                    "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:?}",
                    pc,
                    mem_type,
                    address,
                    size,
                    value
                );
            }
        } else {
            log::trace!(
                "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:x}",
                pc,
                mem_type,
                address,
                size,
                value
            );
        }

        true
    }
}
