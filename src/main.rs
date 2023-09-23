use std::{fs::File, io::Read, str};

use clap::Parser;
use zip::ZipArchive;

use wie_backend::Backend;
use wie_vendor_ktf::KtfWipiModule;

#[derive(Debug)]
struct MidletManifest {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    vendor: Option<String>,
    main_class: Option<String>,
}

impl MidletManifest {
    pub fn parse(manifest: &str) -> Self {
        let mut result = Self {
            name: None,
            description: None,
            version: None,
            vendor: None,
            main_class: None,
        };

        for line in manifest.lines() {
            let split = line.split(':').collect::<Vec<_>>();
            if split.len() < 2 {
                continue;
            }

            let key = split[0];
            let value = split[1].trim();

            match key {
                "MIDlet-Name" => result.name = Some(value.into()),
                "MIDlet-Description" => result.description = Some(value.into()),
                "MIDlet-Version" => result.version = Some(value.into()),
                "MIDlet-Vendor" => result.vendor = Some(value.into()),
                "MIDlet-1" => {
                    let midlet_split = value.split(',').collect::<Vec<_>>();
                    result.main_class = Some(midlet_split[2].trim().into());
                }
                _ => {}
            }
        }

        result
    }
}

enum ArchiveVendor {
    Ktf { module_file_name: String, main_class_name: String },
}

impl ArchiveVendor {
    pub fn from_archive(archive: &mut ZipArchive<File>) -> anyhow::Result<Option<ArchiveVendor>> {
        let manifest = {
            let mut manifest_file = archive.by_name("META-INF/MANIFEST.MF")?;
            let mut manifest = Vec::new();
            manifest_file.read_to_end(&mut manifest)?;

            MidletManifest::parse(str::from_utf8(&manifest)?)
        };
        tracing::info!("Manifest {:?}", manifest);

        let file_names = archive.file_names();

        for file_name in file_names {
            if file_name.starts_with("client.bin") {
                tracing::info!("Found ktf archive, module {}", file_name);

                let main_class_name = manifest.main_class.unwrap_or("Clet".into());

                return Ok(Some(ArchiveVendor::Ktf {
                    module_file_name: file_name.into(),
                    main_class_name,
                }));
            }
        }

        Ok(None)
    }
}

#[derive(Parser)]
struct Args {
    filename: String,
    main_class_name: Option<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let args = Args::parse();

    let file = File::open(args.filename)?;
    let mut archive = ZipArchive::new(file)?;

    let vendor = ArchiveVendor::from_archive(&mut archive)?;
    let backend = Backend::new();

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        backend.add_resource(file.name(), data);
    }

    tracing::info!("Starting module");
    let module = match vendor {
        Some(ArchiveVendor::Ktf {
            module_file_name,
            main_class_name,
        }) => {
            let main_class_name = if let Some(x) = args.main_class_name { x } else { main_class_name };
            KtfWipiModule::new(&module_file_name, &main_class_name, &backend)?
        }
        None => return Err(anyhow::anyhow!("Unknown vendor")),
    };

    backend.run(module)?;

    Ok(())
}
