use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use jvm::Result as JvmResult;

use wie_backend::{App, Event, System};
use wie_core_jvm::JvmCore;

pub struct SktApp {
    system: System,
    jar: Vec<u8>,
    main_class_name: Option<String>,
}

impl SktApp {
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
            anyhow::bail!("Main class not found");
        };

        let normalized_class_name = main_class_name.replace('.', "/");
        let main_class = core.jvm().new_class(&normalized_class_name, "()V", []).await?;

        let result: JvmResult<()> = core
            .jvm()
            .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [None.into()])
            .await;
        if let Err(x) = result {
            if !x.to_string().contains("No such method") {
                core.jvm().invoke_virtual(&main_class, "startApp", "()V", []).await?;
                // both startapp exists in wild.. TODO check this elegantly
            } else {
                return Err(x.into());
            }
        }

        Ok(())
    }
}

impl App for SktApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut system = self.system.clone();

        let jar = self.jar.clone();
        let main_class_name = self.main_class_name.clone();

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
