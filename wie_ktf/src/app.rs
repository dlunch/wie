use alloc::{collections::BTreeMap, string::String, vec::Vec};

use wie_backend::{App, Event, System};
use wie_core_arm::{Allocator, ArmCore};

use crate::runtime::KtfJvmSupport;

pub const IMAGE_BASE: u32 = 0x100000;

pub struct KtfApp {
    core: ArmCore,
    system: System,
    main_class_name: Option<String>,
}

impl KtfApp {
    pub fn new(jar: Vec<u8>, additional_files: BTreeMap<String, Vec<u8>>, main_class_name: Option<String>, system: System) -> anyhow::Result<Self> {
        let mut core = ArmCore::new()?;

        system.filesystem().mount_zip(&jar);

        for (path, data) in additional_files {
            let path = path.trim_start_matches("P/");
            system.filesystem().add(path, data.clone());
        }

        Allocator::init(&mut core)?;

        Ok(Self {
            core,
            system,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, main_class_name: Option<String>) -> anyhow::Result<()> {
        let jvm = KtfJvmSupport::init(core, system).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            anyhow::bail!("Main class not found");
        };

        let main_class_name = main_class_name.replace('.', "/");
        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        let _: () = jvm
            .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        Ok(())
    }
}

impl App for KtfApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut system = self.system.clone();
        let main_class_name = self.main_class_name.clone();

        self.system
            .clone()
            .spawn(move || async move { Self::do_start(&mut core, &mut system, main_class_name).await });

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
