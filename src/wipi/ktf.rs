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
        Self::register_functions(&mut emulator);

        Self {
            emulator,
            base_address,
            bss_size,
        }
    }

    pub fn start(&mut self) {
        let wipi_exe = self.emulator.run_function(self.base_address + 1, &[self.bss_size]);

        println!("{}", wipi_exe);
    }

    fn load(emulator: &mut ArmEmulator, data: &[u8], filename: &str) -> (u32, u32) {
        let bss_start = filename.find("client.bin").unwrap() + 10;
        let bss_size = filename[bss_start..].parse::<u32>().unwrap();

        let base_address = emulator.load(data, data.len() + bss_size as usize);

        (base_address, bss_size)
    }

    fn register_functions(emulator: &mut ArmEmulator) {
        emulator.register_function(test);
    }
}

fn test(_: &mut ArmEmulator) -> u32 {
    0
}
