use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{App, System};
use wie_common::Event;
use wie_core_jvm::JvmCore;

pub struct J2MEApp {
    system: System,
    jar: Vec<u8>,
    main_class_name: String,
}

impl J2MEApp {
    pub fn new(main_class_name: &str, jar: Vec<u8>, system: System) -> anyhow::Result<Self> {
        Ok(Self {
            system,
            jar,
            main_class_name: main_class_name.to_string(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar: Vec<u8>, main_class_name: String) -> anyhow::Result<()> {
        let core = JvmCore::new(system).await?;
        core.add_jar(&jar).await?;

        let normalized_class_name = main_class_name.replace('.', "/");
        core.jvm().invoke_static(&normalized_class_name, "startApp", "()V", []).await?;

        Ok(())
    }
}

impl App for J2MEApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut system = self.system.clone();

        let main_class_name = self.main_class_name.clone();
        let jar = self.jar.clone();

        self.system
            .spawn(move || async move { Self::do_start(&mut system, jar, main_class_name).await });

        Ok(())
    }

    fn on_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}
