use crate::emulator::arm::ArmEmulator;

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    emulator: ArmEmulator,
    base_address: u32,
    bss_size: u32,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str) -> Self {
        let mut emulator = ArmEmulator::new();

        let (base_address, bss_size) = Self::load(&mut emulator, data, filename);

        Self {
            emulator,
            base_address,
            bss_size,
        }
    }

    pub fn start(&mut self) {
        let wipi_exe = self.emulator.run_function(self.base_address + 1, &[self.bss_size]);

        log::debug!("Got wipi_exe {:#x}", wipi_exe);

        let param_4 = InitParam4 {
            get_system_function: self.emulator.register_function(Self::get_system_function),
            get_java_function: 0,
        };

        self.emulator.alloc(0x40000000, 0x10000);
        self.emulator.write(0x40000000, param_4);

        let wipi_exe = self.emulator.read::<WipiExe>(wipi_exe);
        let exe_interface = self.emulator.read::<ExeInterface>(wipi_exe.exe_interface_ptr);
        let exe_interface_functions = self.emulator.read::<ExeInterfaceFunctions>(exe_interface.functions_ptr);

        log::debug!("Call init at {:#x}", exe_interface_functions.init);

        self.emulator.run_function(exe_interface_functions.init, &[0, 0, 0, 0, 0x40000000]);

        self.emulator.free(0x40000000, 0x10000);
    }

    fn load(emulator: &mut ArmEmulator, data: &[u8], filename: &str) -> (u32, u32) {
        let bss_start = filename.find("client.bin").unwrap() + 10;
        let bss_size = filename[bss_start..].parse::<u32>().unwrap();

        let base_address = emulator.load(data, data.len() + bss_size as usize);

        (base_address, bss_size)
    }

    fn get_system_function(emulator: &mut ArmEmulator) -> u32 {
        log::debug!("\n{}", emulator.dump_regs());

        0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct InitParam4 {
    get_system_function: u32,
    get_java_function: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct WipiExe {
    exe_interface_ptr: u32,
    name_ptr: u32,
    unk1: u32,
    unk2: u32,
    unk_func_ptr1: u32,
    unk_func_ptr2: u32,
    unk3: u32,
    unk4: u32,
    unk_func_ptr3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterface {
    functions_ptr: u32,
    name_ptr: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    init: u32,
    get_default_dll: u32,
    unk_func_ptr1: u32,
    unk_func_ptr2: u32,
    unk_func_ptr3: u32,
}
