use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use anyhow::Context;

use wie_backend::{App, Archive, Platform, System};

use crate::{app::KtfApp, context::KtfContext};

pub struct KtfArchive {
    jar: Vec<u8>,
    id: String,
    main_class_name: String,
    additional_files: BTreeMap<String, Vec<u8>>,
}

impl KtfArchive {
    pub fn is_ktf_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("__adf__")
    }

    pub fn from_zip(mut files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let adf = files.get("__adf__").context("Invalid format")?;
        let adf = KtfAdf::parse(adf);

        tracing::info!("Loading app {}, mclass {}", adf.aid, adf.mclass);

        let jar = files.remove(&format!("{}.jar", adf.aid)).context("Invalid format")?;

        let additional_files = files.into_iter().filter(|x| x.0.starts_with("P/")).collect();

        Ok(Self::from_jar(jar, adf.aid, adf.mclass, additional_files))
    }

    pub fn from_jar(data: Vec<u8>, id: String, main_class_name: String, additional_files: BTreeMap<String, Vec<u8>>) -> Self {
        Self {
            jar: data,
            id,
            main_class_name,
            additional_files,
        }
    }
}

impl Archive for KtfArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform, Box::new(KtfContext::new()));

        system.resource_mut().mount_zip(&self.jar)?;

        for (path, data) in self.additional_files {
            let path = path.trim_start_matches("P/");
            system.resource_mut().add(path, data.clone());
        }

        Ok(Box::new(KtfApp::new(&self.main_class_name, system)?))
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
