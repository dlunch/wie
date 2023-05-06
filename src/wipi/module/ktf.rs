mod runtime;

use crate::{
    backend::Backend,
    core::arm::{allocator::Allocator, ArmCore},
    wipi::java::JavaContextBase,
};

use self::runtime::KtfJavaContext;

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    ptr_main_class: u32,
    backend: Backend,
}

impl KtfWipiModule {
    pub fn new(data: &[u8], filename: &str, main_class: &str, backend: Backend) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let ptr_main_class = Self::init(&mut core, &backend, base_address, bss_size, main_class)?;

        Ok(Self {
            core,
            ptr_main_class,
            backend,
        })
    }

    pub fn start(self) -> anyhow::Result<()> {
        let mut java_context = KtfJavaContext::new(self.core, self.backend);

        let instance = java_context.instantiate_from_ptr_class(self.ptr_main_class)?;
        java_context.call_method(&instance, "<init>", "()V", &[])?;

        log::info!("Main class instance: {:#x}", instance.ptr_instance);

        let arg = java_context.instantiate_array("Ljava/lang/String;", 0)?;
        java_context.call_method(&instance, "startApp", "([Ljava/lang/String;)V", &[arg.ptr_instance])?;

        Ok(())
    }

    fn init(core: &mut ArmCore, backend: &Backend, base_address: u32, bss_size: u32, main_class: &str) -> anyhow::Result<u32> {
        let module = runtime::init(core, backend, base_address, bss_size)?;

        log::info!("Call wipi init at {:#x}", module.fn_init);
        let result = core.run_function(module.fn_init, &[])?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let main_class_name = Allocator::alloc(core, 20)?; // TODO size fix
        core.write_raw(main_class_name, main_class.as_bytes())?;

        log::info!("Call class getter at {:#x}", module.fn_get_class);
        let ptr_main_class = core.run_function(module.fn_get_class, &[main_class_name])?;
        if ptr_main_class == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }
        Allocator::free(core, main_class_name)?;

        log::info!("Got main class: {:#x}", ptr_main_class);

        Ok(ptr_main_class)
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").ok_or_else(|| anyhow::anyhow!("Incorrect filename"))? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        log::info!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}
