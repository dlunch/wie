use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use jvm::{runtime::JavaLangString, ClassInstance};

use wie_backend::{extract_zip, Emulator, Event, Platform, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, WieError};

use crate::runtime::KtfJvmSupport;

pub const IMAGE_BASE: u32 = 0x100000;

pub struct KtfEmulator {
    core: ArmCore,
    system: System,
}

impl KtfEmulator {
    pub fn from_archive(platform: Box<dyn Platform>, files: BTreeMap<String, Vec<u8>>) -> Result<Self> {
        let adf = files.get("__adf__").unwrap();
        let adf = KtfAdf::parse(adf);

        tracing::info!("Loading app {}, mclass {}", adf.aid, adf.mclass);

        let jar_filename = format!("{}.jar", adf.aid);

        Self::load(platform, &jar_filename, &adf.aid, Some(adf.mclass), &files)
    }

    pub fn from_jar(platform: Box<dyn Platform>, jar_filename: &str, jar: Vec<u8>, id: &str, main_class_name: Option<String>) -> Result<Self> {
        let files = [(jar_filename.to_owned(), jar)].into_iter().collect();

        Self::load(platform, jar_filename, id, main_class_name, &files)
    }

    pub fn loadable_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("__adf__")
    }

    pub fn loadable_jar(jar: &[u8]) -> bool {
        let files = extract_zip(jar).unwrap();

        for name in files.keys() {
            if name.starts_with("client.bin") {
                return true;
            }
        }

        false
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

        for (path, data) in files {
            let path = path.trim_start_matches("P/");
            system.filesystem().add(path, data.clone());
        }

        Allocator::init(&mut core)?;

        let mut core_clone = core.clone();
        let mut system_clone = system.clone();
        let jar_filename_clone = jar_filename.to_owned();

        system.spawn(move || async move { Self::start(&mut core_clone, &mut system_clone, jar_filename_clone, main_class_name).await });

        Ok(Self { core, system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn start(core: &mut ArmCore, system: &mut System, jar_filename: String, main_class_name: Option<String>) -> Result<()> {
        let (jvm, class_loader) = KtfJvmSupport::init(core, system, Some(&jar_filename)).await?;

        let main_class_name = if let Some(x) = main_class_name {
            x
        } else {
            return Err(WieError::FatalError("Main class not found".into()));
        };

        let main_class_name = main_class_name.replace('.', "/");

        let main_class_name_java = JavaLangString::from_rust_string(&jvm, &main_class_name).await?;
        let _main_class: Box<dyn ClassInstance> = jvm
            .invoke_virtual(
                &class_loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                (main_class_name_java,),
            )
            .await?;
        // TODO can't we use java/lang/Class above?
        let main_class = jvm.new_class(&main_class_name, "()V", []).await?;

        tracing::debug!("Main class instance: {:?}", &main_class);

        let arg = jvm.instantiate_array("Ljava/lang/String;", 0).await?;
        let _: () = jvm
            .invoke_virtual(&main_class, "startApp", "([Ljava/lang/String;)V", [arg.into()])
            .await?;

        Ok(())
    }
}

impl Emulator for KtfEmulator {
    fn handle_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> Result<()> {
        self.system.tick().map_err(|x| {
            let reg_stack = self.core.dump_reg_stack(IMAGE_BASE);
            match x {
                WieError::FatalError(msg) => WieError::FatalError(format!("{}\n{}", msg, reg_stack)),
                _ => WieError::FatalError(format!("{}\n{}", x, reg_stack)),
            }
        })
    }
}

struct KtfAdf {
    aid: String,
    mclass: String,
}

impl KtfAdf {
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
