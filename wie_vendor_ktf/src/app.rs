use alloc::string::String;

use anyhow::Context;

use wie_backend::Backend;
use wie_base::App;
use wie_core_arm::{Allocator, ArmCore};
use wie_wipi_java::r#impl::org::kwis::msp::lcdui::Jlet;

use crate::runtime::KtfJavaContext;

pub struct KtfWipiApp {
    core: ArmCore,
    backend: Backend,
    base_address: u32,
    bss_size: u32,
    main_class_name: String,
}

impl KtfWipiApp {
    pub fn new(main_class_name: &str, backend: &Backend) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        Allocator::init(&mut core)?;

        let resource = backend.resource();
        let filename = resource.files().find(|x| x.starts_with("client.bin")).context("Invalid archive")?;
        let data = resource.data(resource.id(filename).context("Resource not found")?);

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
    async fn do_start(core: &mut ArmCore, backend: &mut Backend, base_address: u32, bss_size: u32, main_class_name: String) -> anyhow::Result<()> {
        let wipi_exe = crate::runtime::start(core, base_address, bss_size).await?;
        tracing::debug!("Got wipi_exe {:#x}", wipi_exe);

        let fn_init = crate::runtime::init(core, backend, wipi_exe).await?;
        tracing::debug!("Call wipi init at {:#x}", fn_init);

        let result = core.run_function::<u32>(fn_init, &[]).await?;
        anyhow::ensure!(result == 0, "wipi init failed with code {:#x}", result);

        let mut java_context = KtfJavaContext::new(core, backend);

        Jlet::start(&mut java_context, &main_class_name).await?;

        Ok(())
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<(u32, u32)> {
        let bss_start = filename.find("client.bin").context("Incorrect filename")? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        let base_address = core.load(data, data.len() + bss_size as usize)?;

        tracing::debug!("Loaded at {:#x}, size {:#x}, bss {:#x}", base_address, data.len(), bss_size);

        Ok((base_address, bss_size))
    }
}

impl App for KtfWipiApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut backend = self.backend.clone();
        let base_address = self.base_address;
        let bss_size = self.bss_size;
        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut backend, base_address, bss_size, main_class_name).await });

        Ok(())
    }

    fn crash_dump(&self) -> String {
        self.core.dump_reg_stack()
    }
}
