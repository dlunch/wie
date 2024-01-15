use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    format, str,
    string::{String, ToString},
    vec::Vec,
};

use anyhow::Context;

use wie_backend::{App, Archive, Platform, System};

use crate::app::SktApp;

pub struct SktArchive {
    jar: Vec<u8>,
    id: String,
    main_class_name: String,
}

impl SktArchive {
    pub fn is_skt_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn from_zip(mut files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let msd = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let msd = SktMsd::parse(msd.0, msd.1);

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar = files.remove(&format!("{}.jar", msd.id)).context("Invalid format")?;

        Ok(Self::from_jar(jar, &msd.id, &msd.main_class))
    }

    pub fn from_jar(data: Vec<u8>, id: &str, main_class_name: &str) -> Self {
        Self {
            jar: data,
            id: id.into(),
            main_class_name: main_class_name.into(),
        }
    }
}

impl Archive for SktArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform, Box::new(()));

        system.handle().resource_mut().mount_zip(&self.jar)?;

        Ok(Box::new(SktApp::new(&self.main_class_name, system)?))
    }
}

struct SktMsd {
    id: String,
    main_class: String,
}

impl SktMsd {
    pub fn parse(filename: &str, data: &[u8]) -> Self {
        let mut main_class = String::new();
        let mut id = filename[..filename.find('.').unwrap()].into();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"MIDlet-1:") {
                let value = line[10..].split(|x| *x == b',').collect::<Vec<_>>();
                main_class = str::from_utf8(value[2]).unwrap().trim().to_string();
            }
            if line.starts_with(b"DD-ProgName") {
                id = str::from_utf8(&line[12..]).unwrap().trim().to_string();
            }
            // TODO load name, it's in euc-kr..
        }

        Self { id, main_class }
    }
}
