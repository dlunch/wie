use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::btree_map::BTreeMap,
    str,
    string::{String, ToString},
    vec::Vec,
};

use jvm::{Result as JvmResult, runtime::JavaLangString};

use wie_backend::{Emulator, Event, Platform, System};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

pub struct J2MEEmulator {
    system: System,
}

impl J2MEEmulator {
    pub fn from_jad_jar(platform: Box<dyn Platform>, jad: Vec<u8>, jar_filename: String, jar: Vec<u8>) -> Result<Self> {
        let descriptor = J2MEDescriptor::parse(&jad);

        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();
        Self::load(platform, &jar_filename, &descriptor.name, Some(descriptor.main_class_name), &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, jar_filename, None, &files)
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
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
    async fn do_start(system: &mut System, jar_filename: String, main_class_name: Option<String>) -> Result<()> {
        let protos = [wie_midp::get_protos().into()];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &[], RustJavaJvmImplementation).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x.replace('.', "/")
        } else {
            // TODO we need to parse META-INF/MANIFEST.MF for midlet
            return Err(WieError::FatalError("Main class not found".into()));
        };

        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = jvm
            .invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
            .await;

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for J2MEEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
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
