use alloc::{boxed::Box, string::String};

use wie_backend::Backend;
use wie_base::Module;
use wie_core_arm::{Allocator, ArmCore, ArmCoreContext};
use wie_wipi_java::r#impl::org::kwis::msp::lcdui::Jlet;

use crate::runtime::KtfJavaContext;

// client.bin from jar, extracted from ktf phone
pub struct KtfWipiModule {
    core: ArmCore,
    backend: Backend,
    base_address: u32,
    bss_size: u32,
    main_class_name: String,
}

impl KtfWipiModule {
    pub fn new(filename: &str, main_class_name: &str, backend: &Backend) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        Allocator::init(&mut core)?;

        let resource = backend.resource();
        let data = resource.data(resource.id(filename).ok_or(anyhow::anyhow!("Resource not found"))?);

        let (base_address, bss_size) = Self::load(&mut core, data, filename)?;

        Ok(Self {
            core,
            backend: backend.clone(),
            base_address,
            bss_size,
            main_class_name: main_class_name.into(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(&mut self) -> anyhow::Result<()> {
        let wipi_exe = crate::runtime::start(&mut self.core, self.base_address, self.bss_size).await?;
        tracing::debug!("Got wipi_exe {:#x}", wipi_exe);

        let fn_init = crate::runtime::init(&mut self.core, &self.backend, wipi_exe).await?;
        tracing::debug!("Call wipi init at {:#x}", fn_init);

        let result = self.core.run_function::<u32>(fn_init, &[]).await?;
        if result != 0 {
            return Err(anyhow::anyhow!("wipi init failed with code {:#x}", result));
        }

        let mut java_context = KtfJavaContext::new(&mut self.core, &mut self.backend);

        Jlet::start(&mut java_context, &self.main_class_name).await?;

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

#[async_trait::async_trait(?Send)]
impl Module for KtfWipiModule {
    async fn start(&mut self) -> anyhow::Result<()> {
        let stack_base = Allocator::alloc(&mut self.core, 0x1000).unwrap();
        let context = ArmCoreContext::new(stack_base);
        self.core.restore_context(&context);

        self.do_start().await?;

        Allocator::free(&mut self.core, stack_base)?;

        Ok(())
    }

    fn crash_dump(&self) -> String {
        self.core.dump_reg_stack()
    }
}
