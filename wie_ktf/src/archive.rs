use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use anyhow::Context;

use wie_backend::{App, Archive, Backend};

use crate::app::KtfApp;

pub struct KtfArchive {
    jar: Vec<u8>,
    id: String,
    main_class_name: String,
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

        // TODO load resource on P directory

        Ok(Self::from_jar(jar, &adf.aid, &adf.mclass))
    }

    pub fn from_jar(data: Vec<u8>, id: &str, main_class_name: &str) -> Self {
        Self {
            jar: data,
            id: id.into(),
            main_class_name: main_class_name.into(),
        }
    }
}

impl Archive for KtfArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(&self, backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
        backend.mount_zip(&self.jar)?;

        Ok(Box::new(KtfApp::new(&self.main_class_name, backend)?))
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
