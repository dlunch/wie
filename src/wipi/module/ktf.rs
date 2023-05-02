mod context;
mod r#impl;
mod types;

use std::mem::size_of;

use crate::core::arm::{allocator::Allocator, ArmCore};

use self::{
    context::Context,
    r#impl::{get_system_struct, init_unk2, init_unk3},
    types::{ExeInterface, ExeInterfaceFunctions, InitParam4, JavaClassInstance, WipiExe},
};

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    context: Context,
    main_class_instance: u32,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str, main_class: &str) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;
        let context = Context::new(Allocator::new(&mut core)?);

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let main_class_instance = Self::init(&mut core, &context, base_address, bss_size, main_class)?;

        Ok(Self {
            core,
            context,
            main_class_instance,
        })
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let instance = self.core.read::<JavaClassInstance>(self.main_class_instance)?;

        log::info!("instance.ptr_class: {:#x}", instance.ptr_class);

        self.context.borrow_mut().allocator.free(self.main_class_instance);

        Ok(())
    }

    fn init(core: &mut ArmCore, context: &Context, base_address: u32, bss_size: u32, main_class: &str) -> anyhow::Result<u32> {
        let wipi_exe = core.run_function(base_address + 1, &[bss_size])?;

        log::info!("Got wipi_exe {:#x}", wipi_exe);

        let param_4 = InitParam4 {
            fn_get_system_struct: core.register_function(get_system_struct, context)?,
            fn_unk1: 0,
            unk1: 0,
            unk2: 0,
            unk3: 0,
            unk4: 0,
            unk5: 0,
            unk6: 0,
            fn_unk2: core.register_function(init_unk2, context)?,
            unk7: 0,
            unk8: 0,
            fn_unk3: core.register_function(init_unk3, context)?,
        };

        let param4_addr = context
            .borrow_mut()
            .allocator
            .alloc(size_of::<InitParam4>() as u32)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;
        core.write(param4_addr, param_4)?;

        let wipi_exe = core.read::<WipiExe>(wipi_exe)?;
        let exe_interface = core.read::<ExeInterface>(wipi_exe.ptr_exe_interface)?;
        let exe_interface_functions = core.read::<ExeInterfaceFunctions>(exe_interface.ptr_functions)?;

        log::info!("Call init at {:#x}", exe_interface_functions.fn_init);
        let result = core.run_function(exe_interface_functions.fn_init, &[0, 0, 0, 0, param4_addr])?;
        if result != 0 {
            return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
        }

        log::info!("Call wipi init at {:#x}", wipi_exe.fn_init);
        let result = core.run_function(wipi_exe.fn_init, &[])?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let main_class_name = context
            .borrow_mut()
            .allocator
            .alloc(20)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?; // TODO size fix
        core.write_raw(main_class_name, main_class.as_bytes())?;

        log::info!("Call class getter at {:#x}", exe_interface_functions.fn_get_class);
        let main_class = core.run_function(exe_interface_functions.fn_get_class, &[main_class_name])?;
        if main_class == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }
        context.borrow_mut().allocator.free(main_class_name);

        log::info!("Got main class: {:#x}", main_class);

        let instance = Self::instantiate_java_class(core, context, main_class)?;

        log::info!("Main class instance: {:#x}", instance);

        Ok(instance)
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").ok_or_else(|| anyhow::anyhow!("Incorrect filename"))? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        log::info!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }

    fn instantiate_java_class(core: &mut ArmCore, context: &Context, class: u32) -> anyhow::Result<u32> {
        let instance = context
            .borrow_mut()
            .allocator
            .alloc(size_of::<JavaClassInstance>() as u32)
            .ok_or_else(|| anyhow::anyhow!("Failed to allocate"))?;

        core.write(instance, JavaClassInstance { ptr_class: class })?;

        Ok(instance)
    }
}
