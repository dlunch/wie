use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    str,
    string::{String, ToString},
    vec::Vec,
};

use jvm::Result as JvmResult;

use wie_backend::{Emulator, Event, Platform, System};
use wie_jvm_support::JvmSupport;

pub struct SktEmulator {
    system: System,
}

impl SktEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let msd_file = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let msd = SktMsd::parse(msd_file.0, msd_file.1);

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar_filename = msd_file.0.replace(".msd", ".jar");

        Self::load(platform, &jar_filename, &msd.id, Some(msd.main_class), &files)
    }

    pub fn from_jar(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        jar: Vec<u8>,
        id: &str,
        main_class_name: Option<String>,
    ) -> anyhow::Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, &files)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        jar.starts_with(b"\x20\x00\x00\x00\x00\x00\x00\x00")
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> anyhow::Result<Self> {
        let mut system = System::new(platform, id);

        for (filename, data) in files {
            system.filesystem().add(filename, data.clone())
        }

        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();

        system.spawn(move || async move { Self::do_start(&mut system_clone, jar_filename_clone, main_class_name).await });

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(system: &mut System, jar_filename: String, main_class_name: Option<String>) -> anyhow::Result<()> {
        let jvm = JvmSupport::new_jvm(system, &jar_filename).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            anyhow::bail!("Main class not found");
        };

        let normalized_class_name = main_class_name.replace('.', "/");
        let main_class = jvm.new_class(&normalized_class_name, "()V", []).await?;

        let result: JvmResult<()> = if jvm.is_instance(&*main_class, "javax/microedition/midlet/MIDlet").await? {
            jvm.invoke_virtual(&main_class, "startApp", "()V", [None.into()]).await
        } else {
            jvm.invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [None.into()]).await
        };

        if let Err(x) = result {
            anyhow::bail!(JvmSupport::format_err(&jvm, x).await)
        }

        Ok(())
    }
}

impl Emulator for SktEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}

struct SktMsd {
    id: String,
    main_class: String,
}

impl SktMsd {
    pub fn parse(filename: &str, data: &[u8]) -> Self {
        let mut main_class = String::new();
        let mut id = filename[..filename.find('.').unwrap()].into();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"MIDlet-1:") {
                let value = line[10..].split(|x| *x == b',').collect::<Vec<_>>();
                main_class = str::from_utf8(value[2]).unwrap().trim().to_string();
            }
            if line.starts_with(b"DD-ProgName") {
                id = str::from_utf8(&line[12..]).unwrap().trim().to_string();
            }
            // TODO load name, it's in euc-kr..
        }

        Self { id, main_class }
    }
}
