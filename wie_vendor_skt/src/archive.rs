use alloc::{
    boxed::Box,
    collections::BTreeMap,
    format, str,
    string::{String, ToString},
    vec::Vec,
};

use anyhow::Context;

use wie_backend::{extract_zip, App, Archive, Backend};

pub struct SktArchive {
    jar: Vec<u8>,
    main_class_name: String,
}

impl SktArchive {
    pub fn is_skt_archive(files: &BTreeMap<String, Vec<u8>>) -> bool {
        files.iter().any(|x| x.0.ends_with(".msd"))
    }

    pub fn from_zip(mut files: BTreeMap<String, Vec<u8>>) -> anyhow::Result<Self> {
        let msd = files.iter().find(|x| x.0.ends_with(".msd")).unwrap();
        let app_id = msd.0.strip_suffix(".msd").unwrap();
        let msd = SktMsd::parse(msd.1);

        tracing::info!("Loading app {}, mclass {}", app_id, msd.main_class);

        let jar = files.remove(&format!("{}.jar", app_id)).context("Invalid format")?;

        Ok(Self::from_jar(jar, &msd.main_class))
    }

    pub fn from_jar(data: Vec<u8>, main_class_name: &str) -> Self {
        Self {
            jar: data,
            main_class_name: main_class_name.into(),
        }
    }
}

impl Archive for SktArchive {
    fn load_app(&self, backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
        let jar_data = extract_zip(&self.jar)?;

        for (filename, data) in jar_data {
            backend.add_resource(&filename, data);
        }

        todo!("load app {}", self.main_class_name)
    }
}

struct SktMsd {
    main_class: String,
}

impl SktMsd {
    pub fn parse(data: &[u8]) -> Self {
        let mut main_class = String::new();

        let mut lines = data.split(|x| *x == b'\n');

        for line in &mut lines {
            if line.starts_with(b"MIDlet-1:") {
                let value = line[10..].split(|x| *x == b',').collect::<Vec<_>>();
                main_class = str::from_utf8(value[2]).unwrap().trim().to_string();
            }
            // TODO load name, it's in euc-kr..
        }

        Self { main_class }
    }
}
