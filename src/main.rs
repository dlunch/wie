use std::{env, fs};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{Arch, Mode, Permission},
    Unicorn,
};
use unicorn_engine::{
    unicorn_const::{HookType, MemType},
    RegisterARM,
};

const IMAGE_BASE: u64 = 0x100000;
const STACK_BASE: u64 = 0x70000000;
const STACK_SIZE: usize = 0x10000;

fn main() {
    let path = env::args().nth(1).unwrap();

    let mut uc = unicorn_engine::Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN | Mode::THUMB).unwrap();
    uc.add_block_hook(block_hook).unwrap();
    uc.add_mem_hook(HookType::MEM_FETCH_UNMAPPED, 0, 0xffff_ffff_ffff_ffff, mem_hook).unwrap();

    setup(&mut uc, &path);

    dump_regs(&uc);
}

fn setup(uc: &mut Unicorn<'_, ()>, path: &str) -> u32 {
    // from jar, extracted from ktf phone
    let bss_start = path.find("client.bin").unwrap() + 10;
    let bss_size = path[bss_start..].parse::<u64>().unwrap();
    let file = fs::read(path).unwrap();

    uc.mem_map(IMAGE_BASE, round_up(file.len() + bss_size as usize, 0x1000), Permission::ALL)
        .unwrap();
    uc.mem_write(IMAGE_BASE, &file).unwrap();

    uc.mem_map(STACK_BASE, STACK_SIZE, Permission::READ | Permission::WRITE).unwrap();

    uc.reg_write(RegisterARM::CPSR, 0x40000010).unwrap(); // usr32
    uc.reg_write(RegisterARM::SP, STACK_BASE + STACK_SIZE as u64).unwrap();

    uc.reg_write(RegisterARM::R0, bss_size).unwrap();

    // relocation
    uc.emu_start(0x100001, 0, 0, 0).unwrap();

    uc.reg_read(RegisterARM::R0).unwrap() as u32
}

fn round_up(num_to_round: usize, multiple: usize) -> usize {
    if multiple == 0 {
        return num_to_round;
    }

    let remainder = num_to_round % multiple;
    if remainder == 0 {
        num_to_round
    } else {
        num_to_round + multiple - remainder
    }
}

fn dump_regs(uc: &Unicorn<'_, ()>) {
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

    dump_regs(uc);

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
