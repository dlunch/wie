use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use anyhow::Context;

use wie_backend::{extract_zip, App, Archive, Platform, System};

use crate::app::KtfApp;

pub struct KtfArchive {
    jar_filename: String,
    id: String,
    main_class_name: Option<String>,
    files: BTreeMap<String, Vec<u8>>,
}

impl KtfArchive {
    pub fn is_ktf_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("__adf__")
    }

    pub fn is_ktf_jar(jar: &[u8]) -> bool {
        let files = extract_zip(jar).unwrap();

        for name in files.keys() {
            if name.starts_with("client.bin") {
                return true;
            }
        }

        false
    }

    pub fn from_zip(files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let adf = files.get("__adf__").context("Invalid format")?;
        let adf = KtfAdf::parse(adf);

        tracing::info!("Loading app {}, mclass {}", adf.aid, adf.mclass);

        let jar_filename = format!("{}.jar", adf.aid);

        Ok(Self {
            jar_filename,
            id: adf.aid,
            main_class_name: Some(adf.mclass),
            files,
        })
    }

    pub fn from_jar(jar_filename: String, jar: Vec<u8>, id: String, main_class_name: Option<String>) -> Self {
        let files = [(jar_filename.clone(), jar)].into_iter().collect();

        Self {
            jar_filename,
            id,
            main_class_name,
            files,
        }
    }
}

impl Archive for KtfArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform);

        Ok(Box::new(KtfApp::new(self.jar_filename, self.files, self.main_class_name, system)?))
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
