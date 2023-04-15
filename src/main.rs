use std::{env, fs, io};

use capstone::{arch::BuildsCapstone, Capstone};
use unicorn_engine::{
    unicorn_const::{Arch, Mode, Permission},
    Unicorn,
};
use unicorn_engine::{
    unicorn_const::{HookType, MemType},
    RegisterARM,
};

fn main() -> io::Result<()> {
    let filename = env::args().nth(1).unwrap();

    let bss_start = filename.find("client.bin").unwrap() + 10;
    let bss_size = filename[bss_start..].parse::<u64>().unwrap();

    let file = fs::read(filename)?;

    let mut uc = unicorn_engine::Unicorn::new(Arch::ARM, Mode::LITTLE_ENDIAN | Mode::THUMB).unwrap();
    uc.add_block_hook(block_hook).unwrap();
    uc.add_mem_hook(HookType::MEM_FETCH_UNMAPPED, 0, 0xffff_ffff_ffff_ffff, mem_hook).unwrap();

    uc.mem_map(0x100000, 0x50000, Permission::ALL).unwrap();
    uc.mem_write(0x100000, &file).unwrap(); // write code

    uc.mem_map(0x70000000, 0x10000, Permission::READ | Permission::WRITE).unwrap(); // init stack

    uc.reg_write(RegisterARM::CPSR, 0x40000010).unwrap(); // set mode to usr32
    uc.reg_write(RegisterARM::SP, 0x70010000).unwrap(); // stack pointer
    uc.reg_write(RegisterARM::LR, 0x1000).unwrap(); // return address

    uc.reg_write(RegisterARM::R0, bss_size).unwrap(); // bss size?, from filename

    uc.emu_start(0x100001, 0x1000, 0, 0).unwrap(); // odd address for thumb

    dump_regs(&uc);

    Ok(())
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
