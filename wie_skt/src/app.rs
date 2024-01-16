use alloc::string::{String, ToString};

use wie_backend::{App, System, SystemHandle};
use wie_common::Event;
use wie_core_jvm::JvmCore;

pub struct SktApp {
    system: System,
    main_class_name: String,
}

impl SktApp {
    pub fn new(main_class_name: &str, system: System) -> anyhow::Result<Self> {
        Ok(Self {
            system,
            main_class_name: main_class_name.to_string(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut SystemHandle, main_class_name: String) -> anyhow::Result<()> {
        let core = JvmCore::new(system).await?;
        let normalized_class_name = main_class_name.replace('.', "/");

        let main_class = core.jvm().new_class(&normalized_class_name, "()V", []).await?;

        let result: anyhow::Result<()> = core
            .jvm()
            .invoke_virtual(&main_class, &normalized_class_name, "startApp", "([Ljava/lang/String;)V", [None.into()])
            .await;
        if let Err(x) = result {
            if !x.to_string().contains("No such method") {
                core.jvm()
                    .invoke_virtual(&main_class, &normalized_class_name, "startApp", "()V", [])
                    .await?;
                // both startapp exists in wild.. TODO check this elegantly
            } else {
                return Err(x);
            }
        }

        Ok(())
    }
}

impl App for SktApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut system_handle = self.system.handle();

        let main_class_name = self.main_class_name.clone();

        self.system
            .handle()
            .spawn(move || async move { Self::do_start(&mut system_handle, main_class_name).await });

        Ok(())
    }

    fn crash_dump(&self) -> String {
        "".into() // TODO
    }

    fn on_event(&mut self, event: Event) {
        self.system.handle().event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}
