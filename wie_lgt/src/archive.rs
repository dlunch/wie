use alloc::{borrow::ToOwned, boxed::Box, collections::BTreeMap, format, string::String, vec::Vec};

use anyhow::Context;

use wie_backend::{extract_zip, App, Archive, Platform, System};

use crate::app::LgtApp;

pub struct LgtArchive {
    jar: Vec<u8>,
    id: String,
    main_class_name: Option<String>,
}

impl LgtArchive {
    pub fn is_lgt_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.contains_key("app_info")
    }

    pub fn is_lgt_jar(jar: &[u8]) -> bool {
        let files = extract_zip(jar).unwrap();

        files.contains_key("binary.mod")
    }

    pub fn from_zip(mut files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let app_info = files.get("app_info").context("Invalid format")?;
        let app_info = LgtAppInfo::parse(app_info);

        tracing::info!("Loading app {}, mclass {}", app_info.aid, app_info.mclass);

        let jar = files.remove(&format!("{}.jar", app_info.aid)).context("Invalid format")?;

        Ok(Self::from_jar(jar, &app_info.aid, Some(app_info.mclass)))
    }

    pub fn from_jar(data: Vec<u8>, id: &str, main_class_name: Option<String>) -> Self {
        Self {
            jar: data,
            id: id.into(),
            main_class_name,
        }
    }
}

impl Archive for LgtArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform, Box::new(()));

        system.filesystem().mount_zip(&self.jar);

        Ok(Box::new(LgtApp::new(self.main_class_name, system)?))
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
