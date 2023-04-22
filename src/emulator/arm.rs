use std::sync::atomic::{AtomicU32, Ordering};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{Arch, HookType, MemType, Mode, Permission},
    RegisterARM, Unicorn,
};

use crate::util::round_up;

const IMAGE_BASE: u64 = 0x100000;
const STACK_BASE: u64 = 0x70000000;
const STACK_SIZE: usize = 0x10000;
const FUNCTIONS_BASE: u64 = 0x80000000;
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

        self.uc.emu_start(address as u64, 0, 0, 0).unwrap();

        self.uc.reg_read(RegisterARM::R0).unwrap() as u32
    }

    pub fn register_function<F>(&mut self, function: F) -> u32
    where
        F: Fn(&mut Self) -> u32 + 'static,
    {
        let bytes = [0x70, 0x47]; // BX LR
        let address = FUNCTIONS_BASE + FUNCTIONS_COUNT.fetch_add(2, Ordering::SeqCst) as u64;

        self.uc.mem_write(address, &bytes).unwrap();

        self.uc
            .add_code_hook(address, address + 2, move |uc, _, _| {
                let mut new_self = Self::from_uc(Unicorn::try_from(uc.get_handle()).unwrap());

                let ret = function(&mut new_self);

                uc.reg_write(RegisterARM::R0, ret as u64).unwrap();
            })
            .unwrap();

        address as u32 + 1
    }

    pub fn dump_regs(&self) {
        Self::dump_regs_inner(&self.uc)
    }

    fn dump_regs_inner(uc: &Unicorn<'_, ()>) {
        println!(
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
        );
        println!(
            "SB: {:x} SL: {:x} FP: {:x} IP: {:x} SP: {:x} LR: {:x} PC: {:x}",
            uc.reg_read(RegisterARM::SB).unwrap(),
            uc.reg_read(RegisterARM::SL).unwrap(),
            uc.reg_read(RegisterARM::FP).unwrap(),
            uc.reg_read(RegisterARM::IP).unwrap(),
            uc.reg_read(RegisterARM::SP).unwrap(),
            uc.reg_read(RegisterARM::LR).unwrap(),
            uc.reg_read(RegisterARM::PC).unwrap()
        );
        println!("APSR: {:032b}", uc.reg_read(RegisterARM::APSR).unwrap());
    }

    fn block_hook(uc: &mut Unicorn<'_, ()>, address: u64, size: u32) {
        println!("-- address: {:x}, size: {:x}", address, size);
        let insn = uc.mem_read_as_vec(address, size as usize).unwrap();

        let cs = Capstone::new()
            .arm()
            .mode(capstone::arch::arm::ArchMode::Thumb)
            .detail(true)
            .build()
            .unwrap();

        let insns = cs.disasm_all(&insn, address).unwrap();
        for insn in insns.iter() {
            println!("{} {}", insn.mnemonic().unwrap(), insn.op_str().unwrap());
        }
        println!("-- reg");

        Self::dump_regs_inner(uc);

        println!("--");
    }

    fn mem_hook(uc: &mut Unicorn<'_, ()>, mem_type: MemType, address: u64, size: usize, value: i64) -> bool {
        let pc = uc.reg_read(RegisterARM::PC).unwrap();

        if mem_type == MemType::READ {
            let value = uc.mem_read_as_vec(address, size).unwrap();
            if size == 4 {
                let value = u32::from_le_bytes(value.try_into().unwrap());
                println!(
                    "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:x}",
                    pc, mem_type, address, size, value
                );
            } else {
                println!(
                    "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:?}",
                    pc, mem_type, address, size, value
                );
            }
        } else {
            println!(
                "pc: {:x} mem_type: {:?} address: {:x} size: {:x} value: {:x}",
                pc, mem_type, address, size, value
            );
        }

        true
    }
}
