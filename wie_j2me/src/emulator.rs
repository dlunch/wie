use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::btree_map::BTreeMap,
    str,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{Emulator, Event, Platform, System};
use wie_core_jvm::JvmCore;

pub struct J2MEEmulator {
    system: System,
}

impl J2MEEmulator {
    pub fn from_jad_jar(platform: Box<dyn Platform>, jad: Vec<u8>, jar_filename: String, jar: Vec<u8>) -> anyhow::Result<Self> {
        let descriptor = J2MEDescriptor::parse(&jad);

        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();
        Self::load(platform, &jar_filename, &descriptor.name, Some(descriptor.main_class_name), &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>) -> anyhow::Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, jar_filename, None, &files)
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> anyhow::Result<Self> {
        let mut system = System::new(platform, id);

        for (path, data) in files {
            system.filesystem().add(path, data.clone());
        }

        let mut system_clone = system.clone();
        let jar_filename = jar_filename.to_owned();

        system.spawn(move || async move { Self::do_start(&mut system_clone, jar_filename, main_class_name).await });

        Ok(J2MEEmulator { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar_filename: String, main_class_name: Option<String>) -> anyhow::Result<()> {
        let core = JvmCore::new(system, &jar_filename).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            // TODO we need to parse META-INF/MANIFEST.MF for midlet
            anyhow::bail!("Main class not found");
        };

        let normalized_class_name = main_class_name.replace('.', "/");
        let main_class = core.jvm().new_class(&normalized_class_name, "()V", []).await?;

        let result: Result<(), _> = core.jvm().invoke_virtual(&main_class, "startApp", "()V", [None.into()]).await;
        if let Err(x) = result {
            anyhow::bail!(JvmCore::format_err(core.jvm(), x).await)
        }

        Ok(())
    }
}

impl Emulator for J2MEEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}

struct J2MEDescriptor {
    name: String,
    main_class_name: String,
}

impl J2MEDescriptor {
    pub fn parse(data: &[u8]) -> Self {
        let lines = data.split(|x| *x == b'\n');

        let mut name = String::new();
        let mut main_class_name = String::new();

        for line in lines {
            let line = str::from_utf8(line).unwrap().trim();

            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, ':');

            let key = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim();

            match key {
                "MIDlet-Name" => name = value.to_string(),
                "MIDlet-1" => main_class_name = value.split(',').nth(2).unwrap().trim().to_string(),
                _ => {}
            }
        }

        Self { name, main_class_name }
    }
}