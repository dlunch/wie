extern crate alloc;

mod backend;
mod core;
mod util;
mod wipi;

use std::{env, fs::File, io::Read, str};

use zip::ZipArchive;

use self::backend::Backend;

enum ArchiveVendor {
    Ktf { module_file_name: String, main_class_name: String },
}

impl ArchiveVendor {
    pub fn from_archive(archive: &mut ZipArchive<File>) -> anyhow::Result<Option<ArchiveVendor>> {
        let manifest = {
            let mut manifest_file = archive.by_name("META-INF/MANIFEST.MF")?;
            let mut manifest = Vec::new();
            manifest_file.read_to_end(&mut manifest)?;

            manifest
        };

        let file_names = archive.file_names();

        for file_name in file_names {
            if file_name.starts_with("client.bin") {
                let main_class_name = Self::find_main_class_from_manifest(manifest)?.unwrap_or("Clet".into());

                log::info!("Found ktf archive, {}, {}", file_name, main_class_name);

                return Ok(Some(ArchiveVendor::Ktf {
                    module_file_name: file_name.into(),
                    main_class_name,
                }));
            }
        }

        Ok(None)
    }

    fn find_main_class_from_manifest(manifest: Vec<u8>) -> anyhow::Result<Option<String>> {
        let content = str::from_utf8(&manifest)?;

        for line in content.lines() {
            if line.starts_with("MIDlet-1") {
                let value = line.split(':').collect::<Vec<_>>()[1];
                let split = value.split(',').collect::<Vec<_>>();

                return Ok(Some(split[2].into()));
            }
        }

        Ok(None)
    }
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let path = env::args().nth(1).ok_or_else(|| anyhow::anyhow!("No filename argument"))?;

    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let vendor = ArchiveVendor::from_archive(&mut archive)?;
    let mut backend = Backend::new();

    match vendor {
        Some(ArchiveVendor::Ktf {
            module_file_name,
            main_class_name,
        }) => {
            let mut module_file = archive.by_name(&module_file_name)?;
            let mut data = Vec::new();
            module_file.read_to_end(&mut data)?;

            let module = wipi::module::ktf::KtfWipiModule::new(&data, &module_file_name, &main_class_name, backend.clone())?;

            module.start()?;
        }
        None => return Err(anyhow::anyhow!("Unknown vendor")),
    }

    backend.scheduler().run();

    Ok(())
}
