use alloc::{
    borrow::ToOwned,
    boxed::Box,
    collections::BTreeMap,
    str,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{App, Archive, Platform, System};

use crate::app::SktApp;

pub struct SktArchive {
    jar_filename: String,
    id: String,
    main_class_name: Option<String>,
    files: BTreeMap<String, Vec<u8>>,
}

impl SktArchive {
    pub fn is_skt_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn is_skt_jar(jar: &[u8]) -> bool {
        jar.starts_with(b"\x20\x00\x00\x00\x00\x00\x00\x00")
    }

    pub fn from_zip(files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let msd_file = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let msd = SktMsd::parse(msd_file.0, msd_file.1);

        tracing::info!("Loading app {}, mclass {}", msd.id, msd.main_class);

        let jar_filename = msd_file.0.replace(".msd", ".jar");

        Ok(Self {
            jar_filename,
            id: msd.id,
            main_class_name: Some(msd.main_class),
            files,
        })
    }

    pub fn from_jar(jar_filename: String, jar: Vec<u8>, id: &str, main_class_name: Option<String>) -> Self {
        let files = [(jar_filename.clone(), jar)].into_iter().collect();

        Self {
            jar_filename,
            id: id.into(),
            main_class_name,
            files,
        }
    }
}

impl Archive for SktArchive {
    fn id(&self) -> String {
        self.id.to_owned()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform);

        for (filename, data) in self.files {
            system.filesystem().add(&filename, data)
        }

        Ok(Box::new(SktApp::new(self.main_class_name, self.jar_filename, system)?))
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
