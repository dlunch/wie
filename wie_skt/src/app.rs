use alloc::string::String;

use jvm::Result as JvmResult;

use wie_backend::{App, Event, System};
use wie_core_jvm::JvmCore;

pub struct SktApp {
    system: System,
    jar_filename: String,
    main_class_name: Option<String>,
}

impl SktApp {
    pub fn new(main_class_name: Option<String>, jar_filename: String, system: System) -> anyhow::Result<Self> {
        Ok(Self {
            system,
            jar_filename,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar_filename: String, main_class_name: Option<String>) -> anyhow::Result<()> {
        let core = JvmCore::new(system, &jar_filename).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            anyhow::bail!("Main class not found");
        };

        let normalized_class_name = main_class_name.replace('.', "/");
        let main_class = core.jvm().new_class(&normalized_class_name, "()V", []).await?;

        let result: JvmResult<()> = if core.jvm().is_instance(&*main_class, "javax/microedition/midlet/MIDlet").await? {
            core.jvm().invoke_virtual(&main_class, "startApp", "()V", [None.into()]).await
        } else {
            core.jvm()
                .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [None.into()])
                .await
        };

        if let Err(x) = result {
            anyhow::bail!(JvmCore::format_err(core.jvm(), x).await)
        }

        Ok(())
    }
}

impl App for SktApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut system = self.system.clone();

        let jar_filename = self.jar_filename.clone();
        let main_class_name = self.main_class_name.clone();

        self.system
            .spawn(move || async move { Self::do_start(&mut system, jar_filename, main_class_name).await });

        Ok(())
    }

    fn on_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}
