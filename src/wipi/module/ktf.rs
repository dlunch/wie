mod context;
mod r#impl;
mod types;

use std::{cell::RefCell, mem::size_of, rc::Rc};

use crate::core::arm::{allocator::Allocator, ArmCore};

use self::{
    context::{Context, ContextStorage},
    r#impl::{get_system_struct, instantiate_java},
    types::{ExeInterface, ExeInterfaceFunctions, InitParam4, WipiExe},
};

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    base_address: u32,
    bss_size: u32,
    context: Context,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str) -> anyhow::Result<Self> {
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
            fn_instantiate_java: self.core.register_function(instantiate_java, &self.context)?,
        };

        let address = (*self.context).borrow_mut().allocator.alloc(size_of::<InitParam4>() as u32).unwrap();
        self.core.write(address, param_4)?;

        let wipi_exe = self.core.read::<WipiExe>(wipi_exe)?;
        let exe_interface = self.core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
        let exe_interface_functions = self.core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

        log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
        self.core.run_function(exe_interface_functions.fn_init, &[0, 0, 0, 0, 0x40000000])?;

        log::info!("Call wipi init at {:#x}", wipi_exe.fn_init);
        self.core.run_function(wipi_exe.fn_init, &[])?;

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
