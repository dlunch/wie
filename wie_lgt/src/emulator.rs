use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use jvm::runtime::{JavaIoInputStream, JavaLangClassLoader};

use wie_backend::{DefaultTaskRunner, Emulator, Event, Options, Platform, System, extract_zip};
use wie_core_arm::{Allocator, ArmCore};
use wie_jvm_support::{JvmSupport, RustJavaJvmImplementation};
use wie_util::{Result, WieError};

use crate::runtime::init::load_native;

pub struct LgtEmulator {
    core: ArmCore,
    system: System,
}

impl LgtEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>, options: Options) -> Result<Self> {
        let app_info = files.get("app_info").unwrap();
        let app_info = LgtAppInfo::parse(app_info);

        tracing::info!("Loading app {}, mclass {}", app_info.aid, app_info.mclass);

        let jar_filename = format!("{}.jar", app_info.aid);

        Self::load(platform, &jar_filename, &app_info.aid, Some(app_info.mclass), &files, options)
    }

    pub fn from_jar(
        platform: Box<dyn Platform>,
        jar_filename: &str,
        jar: Vec<u8>,
        id: &str,
        main_class_name: Option<String>,
        options: Options,
    ) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, &files, options)
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
        options: Options,
    ) -> Result<Self> {
        let mut core = ArmCore::new(options.enable_gdbserver)?;
        let mut system = System::new(platform, id, DefaultTaskRunner);

        for (filename, data) in files {
            system.filesystem().add(filename, data.clone())
        }

        Allocator::init(&mut core)?;

        let main_class_name = main_class_name.map(|x| x.replace('.', "/"));

        let mut core_clone = core.clone();
        let mut system_clone = system.clone();
        let main_class_name_clone = main_class_name.clone();
        let jar_filename = jar_filename.to_owned();

        system.spawn(async move || Self::do_start(&mut core_clone, &mut system_clone, jar_filename, main_class_name_clone).await);

        Ok(Self { core, system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, system: &mut System, jar_filename: String, _main_class_name: Option<String>) -> Result<()> {
        let protos = [wie_midp::get_protos().into(), wie_wipi_java::get_protos().into()];
        let jvm = JvmSupport::new_jvm(system, Some(&jar_filename), Box::new(protos), &[], RustJavaJvmImplementation).await?; // TODO use lgt's java implementation

        let class_loader = jvm.current_class_loader().await.unwrap();
        let stream = JavaLangClassLoader::get_resource_as_stream(&jvm, &class_loader, "binary.mod")
            .await
            .unwrap()
            .unwrap();

        let binary_mod = JavaIoInputStream::read_until_end(&jvm, &stream).await.unwrap();

        load_native(core, system, &jvm, &binary_mod).await?;

        Ok(())
    }
}

impl Emulator for LgtEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick().map_err(|x| {
            let reg_stack = self.core.dump_reg_stack(0x1000); // TODO: hardcode
            match x {
                WieError::FatalError(msg) => WieError::FatalError(format!("{msg}\n{reg_stack}")),
                _ => WieError::FatalError(format!("{x}\n{reg_stack}")),
            }
        })
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
