use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use wie_backend::{extract_zip, Emulator, Event, Platform, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, WieError};

use crate::runtime::init::load_native;

pub struct LgtEmulator {
    system: System,
}

impl LgtEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> Result<Self> {
        let app_info = files.get("app_info").unwrap();
        let app_info = LgtAppInfo::parse(app_info);

        tracing::info!("Loading app {}, mclass {}", app_info.aid, app_info.mclass);

        let jar_filename = format!("{}.jar", app_info.aid);

        Self::load(platform, &jar_filename, &app_info.aid, Some(app_info.mclass), &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>, id: &str, main_class_name: Option<String>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, &files)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("app_info")
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        let files = extract_zip(jar).unwrap();

        files.contains_key("binary.mod")
    }

    fn load(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let mut core = ArmCore::new()?;
        let mut system = System::new(platform, id);

        for (filename, data) in files {
            system.filesystem().add(filename, data.clone())
        }

        Allocator::init(&mut core)?;

        let main_class_name = main_class_name.map(|x| x.replace('.', "/"));

        let mut system_clone = system.clone();
        let main_class_name_clone = main_class_name.clone();
        let jar_filename = jar_filename.to_owned();

        system.spawn(move || async move { Self::do_start(&mut core, &mut system_clone, jar_filename, main_class_name_clone).await });

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, jar_filename: String, _main_class_name: Option<String>) -> Result<()> {
        let data = {
            let filesystem = system.filesystem();
            let files = extract_zip(filesystem.read(&jar_filename).unwrap()).unwrap(); // TODO classloader
            files.get("binary.mod").unwrap().clone()
        };

        load_native(core, system, &data).await?;

        Err(WieError::Unimplemented("Not yet implemented".into()))
    }
}

impl Emulator for LgtEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick()
    }
}

// almost similar to KtfAdf.. can we merge these?
struct LgtAppInfo {
    aid: String,
    mclass: String,
}

impl LgtAppInfo {
    pub fn parse(data: &[u8]) -> Self {
        let mut aid = String::new();
        let mut mclass = String::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"AID:") {
                aid = String::from_utf8_lossy(&line[4..]).into();
            } else if line.starts_with(b"MClass:") {
                mclass = String::from_utf8_lossy(&line[7..]).into();
            }
            // TODO load name, it's in euc-kr..
        }

        Self { aid, mclass }
    }
}
