use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use elf::{endian::AnyEndian, ElfBytes};

use wie_backend::{extract_zip, Emulator, Event, Platform, System};
use wie_core_arm::{Allocator, ArmCore};
use wie_util::{Result, WieError};

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
        _jar_filename: &str,
        id: &str,
        main_class_name: Option<String>,
        _files: &BTreeMap<String, Vec<u8>>,
    ) -> Result<Self> {
        let mut core = ArmCore::new()?;
        let mut system = System::new(platform, id);

        Allocator::init(&mut core)?;

        let entrypoint = {
            let filesystem = system.filesystem();
            let data = filesystem.read("binary.mod").unwrap();

            Self::load_executable(&mut core, data)?
        };

        let main_class_name = main_class_name.map(|x| x.replace('.', "/"));

        let mut system_clone = system.clone();

        let main_class_name_clone = main_class_name.clone();

        system.spawn(move || async move { Self::do_start(&mut core, &mut system_clone, entrypoint, main_class_name_clone).await });

        Ok(Self { system })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, _system: &mut System, entrypoint: u32, _main_class_name: Option<String>) -> Result<()> {
        let _: () = core.run_function(entrypoint + 1, &[]).await?;

        Err(WieError::Unimplemented("Not yet implemented".into()))
    }

    fn load_executable(core: &mut ArmCore, data: &[u8]) -> Result<u32> {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(data).unwrap();

        assert!(elf.ehdr.e_machine == elf::abi::EM_ARM, "Invalid machine type");
        assert!(elf.ehdr.e_type == elf::abi::ET_EXEC, "Invalid file type");
        assert!(elf.ehdr.class == elf::file::Class::ELF32, "Invalid file type");
        assert!(elf.ehdr.e_phnum == 0, "Invalid file type");

        let (shdrs_opt, strtab_opt) = elf.section_headers_with_strtab().unwrap();
        let (shdrs, strtab) = (shdrs_opt.unwrap(), strtab_opt.unwrap());

        for shdr in shdrs {
            let section_name = strtab.get(shdr.sh_name as usize).unwrap();

            if shdr.sh_addr != 0 {
                tracing::debug!("Section {} at {:x}", section_name, shdr.sh_addr);

                let data = elf.section_data(&shdr).unwrap().0;

                core.load(data, shdr.sh_addr as u32, shdr.sh_size as usize)?;
            }
        }

        tracing::debug!("Entrypoint: {:#x}", elf.ehdr.e_entry);

        Ok(elf.ehdr.e_entry as u32)
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
