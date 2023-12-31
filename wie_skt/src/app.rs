use alloc::string::{String, ToString};

use jvm::JavaValue;

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
    #[allow(clippy::await_holding_refcell_ref)]
    async fn do_start(core: &mut JvmCore, backend: &mut Backend, main_class_name: String) -> anyhow::Result<()> {
        let normalized_class_name = main_class_name.replace('.', "/");

        let result: anyhow::Result<()> = core
            .jvm()
            .invoke_static(&normalized_class_name, "startApp", "([Ljava/lang/String;)V", [JavaValue::Object(None)])
            .await;
        if result.is_err() {
            core.jvm().invoke_static(&normalized_class_name, "startApp", "()V", []).await?;
            // both startapp exists in wild.. TODO check this elegantly
        }

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
        "".into() // TODO
    }
}
