extern crate alloc;

use alloc::{string::String, vec::Vec};
use std::io::{Cursor, Read};

use zip::ZipArchive;

use wie_backend::Backend;
use wie_base::App;
use wie_vendor_ktf::KtfWipiApp;

#[derive(Debug)]
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

fn add_resources_from_zip(zip: &[u8], backend: &mut Backend) -> anyhow::Result<()> {
    let mut archive = ZipArchive::new(Cursor::new(zip))?;

    for index in 0..archive.len() {
        let mut file = archive.by_index(index)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        backend.add_resource(file.name(), data);
    }

    Ok(())
}

fn load_ktf_archive(mut archive: ZipArchive<Cursor<&[u8]>>, backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
    let mut adf = Vec::new();
    archive.by_name("__adf__")?.read_to_end(&mut adf)?;

    let adf = KtfAdf::parse(&adf);

    tracing::info!("Loading app {}, mclass {}", adf.aid, adf.mclass);

    let jar_filename = format!("{}.jar", adf.aid);
    let mut jar = Vec::new();
    archive.by_name(&jar_filename)?.read_to_end(&mut jar)?;

    add_resources_from_zip(&jar, backend)?;

    Ok(Box::new(KtfWipiApp::new(&adf.mclass, backend)?))
}

fn load_archive(file: &[u8], backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
    let mut archive = ZipArchive::new(Cursor::new(file))?;

    for index in 0..archive.len() {
        let file = archive.by_index(index)?;

        if file.name() == "__adf__" {
            drop(file);

            return load_ktf_archive(archive, backend);
        }
    }

    anyhow::bail!("Unknown vendor")
}

pub fn start(file: &[u8]) -> anyhow::Result<()> {
    let mut backend = Backend::new();

    let app = load_archive(file, &mut backend)?;

    backend.run(app)?;

    Ok(())
}
