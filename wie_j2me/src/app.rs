use alloc::string::{String, ToString};

use wie_backend::{App, System};
use wie_core_jvm::JvmCore;

pub struct J2MEApp {
    core: JvmCore,
    system: System,
    main_class_name: String,
}

impl J2MEApp {
    pub fn new(main_class_name: &str, system: &System) -> anyhow::Result<Self> {
        let core = JvmCore::new(system);

        Ok(Self {
            core,
            system: system.clone(),
            main_class_name: main_class_name.to_string(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    #[allow(unused_variables)]
    #[allow(clippy::await_holding_refcell_ref)]
    async fn do_start(core: &mut JvmCore, system: &mut System, main_class_name: String) -> anyhow::Result<()> {
        let normalized_class_name = main_class_name.replace('.', "/");

        core.jvm().invoke_static(&normalized_class_name, "startApp", "()V", []).await?;

        Ok(())
    }
}

impl App for J2MEApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut system_clone = self.system.clone();

        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut system_clone, main_class_name).await });

        Ok(())
    }

    fn crash_dump(&self) -> String {
        todo!()
    }
}
