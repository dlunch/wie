use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{App, System, SystemHandle};
use wie_common::Event;
use wie_core_jvm::JvmCore;

pub struct SktApp {
    system: System,
    jar: Vec<u8>,
    main_class_name: String,
}

impl SktApp {
    pub fn new(main_class_name: &str, jar: Vec<u8>, system: System) -> anyhow::Result<Self> {
        Ok(Self {
            system,
            jar,
            main_class_name: main_class_name.to_string(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut SystemHandle, jar: Vec<u8>, main_class_name: String) -> anyhow::Result<()> {
        let core = JvmCore::new(system).await?;
        core.add_jar(&jar).await?;

        let normalized_class_name = main_class_name.replace('.', "/");
        let main_class = core.jvm().new_class(&normalized_class_name, "()V", []).await?;

        let result: anyhow::Result<()> = core
            .jvm()
            .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [None.into()])
            .await;
        if let Err(x) = result {
            if !x.to_string().contains("No such method") {
                core.jvm().invoke_virtual(&main_class, "startApp", "()V", []).await?;
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

        let jar = self.jar.clone();
        let main_class_name = self.main_class_name.clone();

        self.system
            .handle()
            .spawn(move || async move { Self::do_start(&mut system_handle, jar, main_class_name).await });

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
