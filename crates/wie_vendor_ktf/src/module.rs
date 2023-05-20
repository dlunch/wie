use alloc::string::String;

use wie_backend::{Backend, Task};
use wie_base::util::ByteWrite;
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::JavaContextBase;

use crate::{runtime::KtfJavaContext, task::KtfTask};

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {}

impl KtfWipiModule {
    pub fn start(data: &[u8], filename: &str, main_class_name: &str, backend: Backend) -> anyhow::Result<impl Task> {
        let mut core = ArmCore::new()?;
        Allocator::init(&mut core)?;

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let ptr_main_class_name = Allocator::alloc(&mut core, 20)?; // TODO size fix
        core.write_bytes(ptr_main_class_name, main_class_name.as_bytes())?;

        let entry = core.register_function(Self::do_start, &backend)?;

        let task = KtfTask::from_pc_args(&mut core, entry, &[base_address, bss_size, ptr_main_class_name])?;

        Ok(task)
    }

    fn do_start(mut core: ArmCore, mut backend: Backend, base_address: u32, bss_size: u32, main_class_name: String) -> anyhow::Result<u32> {
        let ptr_main_class = Self::init(&mut core, &backend, base_address, bss_size, &main_class_name)?;

        let mut java_context = KtfJavaContext::new(&mut core, &mut backend);

        let instance = java_context.instantiate_from_ptr_class(ptr_main_class)?;
        java_context.call_method(&instance, "<init>", "()V", &[])?;

        log::info!("Main class instance: {:#x}", instance.ptr_instance);

        let arg = java_context.instantiate_array("Ljava/lang/String;", 0)?;
        java_context.call_method(&instance, "startApp", "([Ljava/lang/String;)V", &[arg.ptr_instance])?;

        Ok(0)
    }

    fn init(core: &mut ArmCore, backend: &Backend, base_address: u32, bss_size: u32, main_class_name: &str) -> anyhow::Result<u32> {
        let module = crate::runtime::init(core, backend, base_address, bss_size)?;

        log::info!("Call wipi init at {:#x}", module.fn_init);
        let result = core.run_function(module.fn_init, &[])?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let ptr_main_class_name = Allocator::alloc(core, 20)?; // TODO size fix
        core.write_bytes(ptr_main_class_name, main_class_name.as_bytes())?;

        log::info!("Call class getter at {:#x}", module.fn_get_class);
        let ptr_main_class = core.run_function(module.fn_get_class, &[ptr_main_class_name])?;
        if ptr_main_class == 0 {
            return Err(anyhow::anyhow!("Failed to get main class"));
        }
        Allocator::free(core, ptr_main_class_name)?;

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
