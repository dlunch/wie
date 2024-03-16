use alloc::{string::String, vec::Vec};

use wie_backend::{App, Event, System};
use wie_core_jvm::JvmCore;

pub struct J2MEApp {
    system: System,
    jar: Vec<u8>,
    main_class_name: Option<String>,
}

impl J2MEApp {
    pub fn new(main_class_name: Option<String>, jar: Vec<u8>, system: System) -> anyhow::Result<Self> {
        Ok(Self {
            system,
            jar,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar: Vec<u8>, main_class_name: Option<String>) -> anyhow::Result<()> {
        let core = JvmCore::new(system).await?;
        let jar_main_class = core.add_jar(&jar).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else if let Some(x) = jar_main_class {
            x
        } else {
            // TODO we need to parse META-INF/MANIFEST.MF for midlet
            anyhow::bail!("Main class not found");
        };

        let normalized_class_name = main_class_name.replace('.', "/");
        let result: Result<(), _> = core.jvm().invoke_static(&normalized_class_name, "startApp", "()V", []).await;
        if let Err(x) = result {
            anyhow::bail!(JvmCore::format_err(core.jvm(), x).await)
        }

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
