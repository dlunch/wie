mod context;
mod r#impl;
mod types;

use std::{cell::RefCell, mem::size_of, rc::Rc};

use crate::core::arm::{allocator::Allocator, ArmCore};

use self::{
    context::{Context, ContextStorage},
    r#impl::{get_system_struct, init_unk1, init_unk2},
    types::{ExeInterface, ExeInterfaceFunctions, InitParam4, WipiExe},
};

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    base_address: u32,
    bss_size: u32,
    context: Context,
    main_class: String,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str, main_class: &str) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let context = Rc::new(RefCell::new(ContextStorage {
            allocator: Allocator::new(&mut core)?,
        }));

        Ok(Self {
            core,
            base_address,
            bss_size,
            context,
            main_class: main_class.into(),
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let wipi_exe = self.core.run_function(self.base_address + 1, &[self.bss_size])?;

        log::info!("Got wipi_exe {:#x}", wipi_exe);

        let param_4 = InitParam4 {
            fn_get_system_struct: self.core.register_function(get_system_struct, &self.context)?,
            fn_get_java_function: 0x12341234,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            unk4: 0,
            unk5: 0,
            unk6: 0,
            fn_unk1: self.core.register_function(init_unk1, &self.context)?,
            unk7: 0,
            unk8: 0,
            fn_unk2: self.core.register_function(init_unk2, &self.context)?,
        };

        let param4_addr = (*self.context).borrow_mut().allocator.alloc(size_of::<InitParam4>() as u32).unwrap();
        self.core.write(param4_addr, param_4)?;

        let wipi_exe = self.core.read::<WipiExe>(wipi_exe)?;
        let exe_interface = self.core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
        let exe_interface_functions = self.core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

        log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
        let result = self.core.run_function(exe_interface_functions.fn_init, &[0, 0, 0, 0, param4_addr])?;
        if result != 0 {
            return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
        }

        log::info!("Call wipi init at {:#x}", wipi_exe.fn_init);
        let result = self.core.run_function(wipi_exe.fn_init, &[])?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let address = (*self.context).borrow_mut().allocator.alloc(20).unwrap(); // TODO size fix
        self.core.write_raw(address, self.main_class.as_bytes())?;

        let result = self.core.run_function(exe_interface_functions.fn_set_main_class, &[address])?;
        if result == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }

        log::info!("Got main class: {:#x}", result);

        Ok(())
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").unwrap() + 10;
        let bss_size = filename[bss_start..].parse::<u32>().unwrap();

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        log::info!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}
