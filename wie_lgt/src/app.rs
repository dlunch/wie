use alloc::string::String;

use anyhow::Context;
use elf::{endian::AnyEndian, ElfBytes};

use wie_backend::{App, System, SystemHandle};
use wie_base::Event;
use wie_core_arm::{Allocator, ArmCore};

pub struct LgtApp {
    core: ArmCore,
    system: System,
    entrypoint: u32,
    main_class_name: String,
}

impl LgtApp {
    pub fn new(main_class_name: &str, system: System) -> anyhow::Result<Self> {
        let system_handle = system.handle();

        let mut core = ArmCore::new(system_handle.clone())?;

        Allocator::init(&mut core)?;

        let resource = system_handle.resource();
        let data = resource.data(resource.id("binary.mod").context("Resource not found")?);

        let entrypoint = Self::load(&mut core, data)?;

        let main_class_name = main_class_name.replace('.', "/");

        Ok(Self {
            core,
            system,
            entrypoint,
            main_class_name,
        })
    }

    #[tracing::instrument(name = "start", skip_all)]
    #[allow(unused_variables)]
    async fn do_start(core: &mut ArmCore, system: &mut SystemHandle, entrypoint: u32, main_class_name: String) -> anyhow::Result<()> {
        core.run_function(entrypoint + 1, &[]).await?;

        todo!()
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
        let mut system_handle = self.system.handle();

        let entrypoint = self.entrypoint;
        let main_class_name = self.main_class_name.clone();

        self.core
            .spawn(move || async move { Self::do_start(&mut core, &mut system_handle, entrypoint, main_class_name).await });

        Ok(())
    }

    fn crash_dump(&self) -> String {
        self.core.dump_reg_stack(0)
    }

    fn on_event(&mut self, event: Event) {
        self.system.handle().event_queue().push(event)
    }

    fn tick(&mut self) -> anyhow::Result<()> {
        self.system.tick()
    }
}
