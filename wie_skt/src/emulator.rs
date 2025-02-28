use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    format, str,
    string::{String, ToString},
    vec,
    vec::Vec,
};

use jvm::{Result as JvmResult, runtime::JavaLangString};

use wie_backend::{DefaultTaskRunner, Emulator, Event, Platform, System};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

pub struct SktEmulator {
    system: System,
}

impl SktEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> Result<Self> {
        let msd_file = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let msd = SktMsd::parse(msd_file.0, msd_file.1);

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar_filename = msd_file.0.replace(".msd", ".jar");

        Self::load(platform, &jar_filename, &msd.id, Some(msd.main_class), msd.properties, &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>, id: &str, main_class_name: Option<String>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, BTreeMap::new(), &files)
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
        properties: BTreeMap<String, String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let mut system = System::new(platform, id, DefaultTaskRunner);

        for (filename, data) in files {
            system.filesystem().add(filename, data.clone())
        }

        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();

        system.spawn(async move || Self::do_start(&mut system_clone, jar_filename_clone, properties, main_class_name).await);

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(
        system: &mut System,
        jar_filename: String,
        properties: BTreeMap<String, String>,
        main_class_name: Option<String>,
    ) -> Result<()> {
        let system_properties = [
            ("MIN", "01000000000"),
            ("m.MIN", "01000000000"),
            ("m.COLOR", "7"),
            ("m.VENDER", "vender"),
            ("m.CARRIER", "SKT"),
            ("m.SK_VM", "10"),
            ("com.xce.wipi.version", ""),
        ];
        let properties = properties
            .into_iter()
            .map(|(k, v)| (format!("wie.appProperty.{}", k), v))
            .collect::<Vec<_>>();
        let properties = system_properties
            .into_iter()
            .chain(properties.iter().map(|(k, v)| (k.as_ref(), v.as_ref())))
            .collect::<Vec<_>>();

        let protos = [
            wie_midp::get_protos().into(),
            wie_skvm::get_protos().into(),
            wie_wipi_java::get_protos().into(),
        ];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &properties, RustJavaJvmImplementation).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x.replace('.', "/")
        } else {
            return Err(WieError::FatalError("Main class not found".into()))?;
        };

        let main_class = jvm.resolve_class(&main_class_name).await.unwrap();
        let main_class_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await.unwrap();

        let result: JvmResult<()> = if jvm.is_inherited_from(&*main_class.definition, "javax/microedition/midlet/MIDlet") {
            jvm.invoke_static("net/wie/Launcher", "start", "(Ljava/lang/String;)V", (main_class_java,))
                .await
        } else {
            let mut args = jvm.instantiate_array("Ljava/lang/String;", 1).await.unwrap();
            jvm.store_array(&mut args, 0, vec![main_class_java]).await.unwrap();
            jvm.invoke_static("org/kwis/msp/lcdui/Main", "main", "([Ljava/lang/String;)V", (args,))
                .await
        };

        if let Err(x) = result {
            return Err(JvmSupport::to_wie_err(&jvm, x).await);
        }

        Ok(())
    }
}

impl Emulator for SktEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick()
    }
}

struct SktMsd {
    id: String,
    main_class: String,
    properties: BTreeMap<String, String>,
}

impl SktMsd {
    pub fn parse(filename: &str, data: &[u8]) -> Self {
        let mut main_class = String::new();
        let mut id = filename[..filename.find('.').unwrap()].into();
        let mut properties = BTreeMap::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"MIDlet-1:") {
                let value = line[10..].split(|x| *x == b',').collect::<Vec<_>>();
                main_class = str::from_utf8(value[2]).unwrap().trim().to_string();
            }
            if line.starts_with(b"DD-ProgName") {
                id = str::from_utf8(&line[12..]).unwrap().trim().to_string();
            }

            let sep = line.iter().position(|x| *x == b':');
            if let Some(sep) = sep {
                let key = &line[..sep];
                let value = &line[sep + 1..];

                if let (Ok(key), Ok(value)) = (str::from_utf8(key), str::from_utf8(value)) {
                    tracing::info!("Adding property {}={}", key.trim(), value.trim());
                    properties.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }

        Self { id, main_class, properties }
    }
}
