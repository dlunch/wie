use alloc::{boxed::Box, format, string::String};

use anyhow::Context;

use wie_backend::Backend;
use wie_base::App;

use crate::app::KtfWipiApp;

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

pub fn is_ktf_archive_loaded(backend: &mut Backend) -> bool {
    backend.resource().id("__adf__").is_some()
}

pub fn load_ktf_archive(backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
    let resource = backend.resource();
    let adf = resource.data(resource.id("__adf__").context("Invalid format")?);

    let adf = KtfAdf::parse(adf);

    tracing::info!("Loading app {}, mclass {}", adf.aid, adf.mclass);

    let jar_filename = format!("{}.jar", adf.aid);
    let jar = resource.data(resource.id(&jar_filename).context("Invalid format")?).to_vec();
    drop(resource);

    backend.add_resources_from_zip(&jar)?;

    Ok(Box::new(KtfWipiApp::new(&adf.mclass, backend)?))
}
