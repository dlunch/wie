use alloc::string::String;

use futures::FutureExt;

use wie_backend::Backend;
use wie_base::{util::ByteWrite, Module};
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::r#impl::org::kwis::msp::lcdui::Jlet;

use crate::runtime::KtfJavaContext;

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    entry: u32,
    base_address: u32,
    bss_size: u32,
    ptr_main_class_name: u32,
}

impl KtfWipiModule {
    pub fn new(filename: &str, main_class_name: &str, backend: &Backend) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        Allocator::init(&mut core)?;

        let resource = backend.resource();
        let data = resource.data(resource.id(filename).ok_or(anyhow::anyhow!("Resource not found"))?);

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        let ptr_main_class_name = Allocator::alloc(&mut core, 20)?; // TODO size fix
        core.write_bytes(ptr_main_class_name, main_class_name.as_bytes())?;

        let entry = core.register_function(Self::do_start, backend)?;

        Ok(Self {
            core,
            entry,
            base_address,
            bss_size,
            ptr_main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, backend: &mut Backend, base_address: u32, bss_size: u32, main_class_name: String) -> anyhow::Result<()> {
        let wipi_exe = crate::runtime::start(core, base_address, bss_size).await?;
        tracing::debug!("Got wipi_exe {:#x}", wipi_exe);

        let fn_init = crate::runtime::init(core, backend, wipi_exe).await?;
        tracing::debug!("Call wipi init at {:#x}", fn_init);

        let result = core.run_function::<u32>(fn_init, &[]).await?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let mut java_context = KtfJavaContext::new(core, backend);

        Jlet::start(&mut java_context, &main_class_name).await?;

        Ok(())
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").ok_or_else(|| anyhow::anyhow!("Incorrect filename"))? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        tracing::debug!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}

impl Module for KtfWipiModule {
    fn start(&mut self) {
        let entry = self.entry;
        let args = [self.base_address, self.bss_size, self.ptr_main_class_name];

        let mut core = self.core.clone();

        self.core.spawn(move || core.run_function::<()>(entry, &args).boxed_local())
    }

    fn crash_dump(&self) -> String {
        self.core.dump_reg_stack()
    }
}
