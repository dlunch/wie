use alloc::string::{String, ToString};

use wie_backend::{App, Backend};
use wie_core_jvm::JvmCore;

pub struct SktApp {
    core: JvmCore,
    backend: Backend,
    main_class_name: String,
}

impl SktApp {
    pub fn new(main_class_name: &str, backend: &Backend) -> anyhow::Result<Self> {
        let core = JvmCore::new(backend);

        Ok(Self {
            core,
            backend: backend.clone(),
            main_class_name: main_class_name.to_string(),
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    #[allow(unused_variables)]
    async fn do_start(core: &mut JvmCore, backend: &mut Backend, main_class_name: String) -> anyhow::Result<()> {
        core.invoke_static_method(&main_class_name, "startApp", "([Ljava/lang/String;)V")?;

        Ok(())
    }
}

impl App for SktApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut backend = self.backend.clone();

        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut backend, main_class_name).await });

        Ok(())
    }

    fn crash_dump(&self) -> String {
        todo!()
    }
}
