use alloc::{collections::BTreeMap, string::String, vec::Vec};

use anyhow::Context;

use wie_backend::{App, Event, System};
use wie_core_arm::{Allocator, ArmCore};

use crate::context::KtfContextExt;

const IMAGE_BASE: u32 = 0x100000;

pub struct KtfApp {
    core: ArmCore,
    system: System,
    bss_size: u32,
    main_class_name: Option<String>,
}

impl KtfApp {
    pub fn new(jar: Vec<u8>, additional_files: BTreeMap<String, Vec<u8>>, main_class_name: Option<String>, system: System) -> anyhow::Result<Self> {
        let mut core = ArmCore::new(system.clone())?;

        system.resource_mut().mount_zip(&jar)?;

        for (path, data) in additional_files {
            let path = path.trim_start_matches("P/");
            system.resource_mut().add(path, data.clone());
        }

        Allocator::init(&mut core)?;

        let bss_size = {
            let resource = system.resource();
            let filename = resource.files().find(|x| x.starts_with("client.bin")).context("Invalid archive")?;
            let data = resource.data(resource.id(filename).context("Resource not found")?);

            Self::load(&mut core, data, filename)?
        };

        Ok(Self {
            core,
            system,
            bss_size,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, bss_size: u32, main_class_name: Option<String>) -> anyhow::Result<()> {
        // we should reverse the order of initialization
        // jvm should go first, and we load client.bin from jvm classloader on init

        let wipi_exe = crate::runtime::start(core, IMAGE_BASE, bss_size).await?;
        tracing::debug!("Got wipi_exe {:#x}", wipi_exe);

        let fn_init = crate::runtime::init(core, system, wipi_exe).await?;
        tracing::debug!("Call wipi init at {:#x}", fn_init);

        let result = core.run_function::<u32>(fn_init, &[]).await?;
        anyhow::ensure!(result == 0, "wipi init failed with code {:#x}", result);

        let jvm = system.jvm();

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            anyhow::bail!("Main class not found");
        };

        let main_class_name = main_class_name.replace('.', "/");
        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        jvm.invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        Ok(())
    }

    fn load(core: &mut ArmCore, data: &[u8], filename: &str) -> anyhow::Result<u32> {
        let bss_start = filename.find("client.bin").context("Incorrect filename")? + 10;
        let bss_size = filename[bss_start..].parse::<u32>()?;

        core.load(data, IMAGE_BASE, data.len() + bss_size as usize)?;

        tracing::debug!("Loaded at {:#x}, size {:#x}, bss {:#x}", IMAGE_BASE, data.len(), bss_size);

        Ok(bss_size)
    }
}

impl App for KtfApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut system = self.system.clone();

        let bss_size = self.bss_size;
        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut system, bss_size, main_class_name).await });

        Ok(())
    }

    fn on_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system
            .tick()
            .map_err(|x| anyhow::anyhow!("{}\n{}", x, self.core.dump_reg_stack(IMAGE_BASE)))
    }
}
