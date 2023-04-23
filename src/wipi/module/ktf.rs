mod r#impl;
mod types;

use crate::emulator::arm::ArmEmulator;

use r#impl::get_system_struct;
use types::{ExeInterface, ExeInterfaceFunctions, InitParam4, WipiExe};

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

        log::info!("Got wipi_exe {:#x}", wipi_exe);

        let param_4 = InitParam4 {
            get_system_struct_fn: self.emulator.register_function(get_system_struct),
            get_java_function_fn: 0,
        };

        self.emulator.alloc(0x40000000, 0x10000);
        self.emulator.write(0x40000000, param_4);

        let wipi_exe = self.emulator.read::<WipiExe>(wipi_exe);
        let exe_interface = self.emulator.read::<ExeInterface>(wipi_exe.exe_interface_ptr);
        let exe_interface_functions = self.emulator.read::<ExeInterfaceFunctions>(exe_interface.functions_ptr);

        log::info!("Call init at {:#x}", exe_interface_functions.init_fn);

        self.emulator.run_function(exe_interface_functions.init_fn, &[0, 0, 0, 0, 0x40000000]);

        self.emulator.free(0x40000000, 0x10000);
    }

    fn load(emulator: &mut ArmEmulator, data: &[u8], filename: &str) -> (u32, u32) {
        let bss_start = filename.find("client.bin").unwrap() + 10;
        let bss_size = filename[bss_start..].parse::<u32>().unwrap();

        let base_address = emulator.load(data, data.len() + bss_size as usize);

        log::info!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        (base_address, bss_size)
    }
}
