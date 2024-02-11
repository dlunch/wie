use alloc::string::String;

use anyhow::Context;
use elf::{endian::AnyEndian, ElfBytes};

use wie_backend::{App, Event, System};
use wie_core_arm::{Allocator, ArmCore};

pub struct LgtApp {
    core: ArmCore,
    system: System,
    entrypoint: u32,
    main_class_name: String,
}

impl LgtApp {
    pub fn new(main_class_name: &str, system: System) -> anyhow::Result<Self> {
        let mut core = ArmCore::new(system.clone())?;

        Allocator::init(&mut core)?;

        let entrypoint = {
            let resource = system.resource();
            let data = resource.data(resource.id("binary.mod").context("Resource not found")?);

            Self::load(&mut core, data)?
        };

        let main_class_name = main_class_name.replace('.', "/");

        Ok(Self {
            core,
            system,
            entrypoint,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    async fn do_start(core: &mut ArmCore, _system: &mut System, entrypoint: u32, _main_class_name: String) -> anyhow::Result<()> {
        core.run_function(entrypoint + 1, &[]).await?;

        anyhow::bail!("Not yet implemented")
    }

    fn load(core: &mut ArmCore, data: &[u8]) -> anyhow::Result<u32> {
        let elf = ElfBytes::<AnyEndian>::minimal_parse(data)?;

        anyhow::ensure!(elf.ehdr.e_machine == elf::abi::EM_ARM, "Invalid machine type");
        anyhow::ensure!(elf.ehdr.e_type == elf::abi::ET_EXEC, "Invalid file type");
        anyhow::ensure!(elf.ehdr.class == elf::file::Class::ELF32, "Invalid file type");
        anyhow::ensure!(elf.ehdr.e_phnum == 0, "Invalid file type");

        let (shdrs_opt, strtab_opt) = elf.section_headers_with_strtab()?;
        let (shdrs, strtab) = (
            shdrs_opt.ok_or(anyhow::anyhow!("Invalid file"))?,
            strtab_opt.ok_or(anyhow::anyhow!("Invalid file"))?,
        );

        for shdr in shdrs {
            let section_name = strtab.get(shdr.sh_name as usize)?;

            if shdr.sh_addr != 0 {
                tracing::debug!("Section {} at {:x}", section_name, shdr.sh_addr);

                let data = elf.section_data(&shdr)?.0;

                core.load(data, shdr.sh_addr as u32, shdr.sh_size as usize)?;
            }
        }

        tracing::debug!("Entrypoint: {:#x}", elf.ehdr.e_entry);

        Ok(elf.ehdr.e_entry as u32)
    }
}

impl App for LgtApp {
    fn start(&mut self) -> anyhow::Result<()> {
        let mut core = self.core.clone();
        let mut system = self.system.clone();

        let entrypoint = self.entrypoint;
        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut system, entrypoint, main_class_name).await });

        Ok(())
    }

    fn on_event(&mut self, event: Event) {
        self.system.event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}
