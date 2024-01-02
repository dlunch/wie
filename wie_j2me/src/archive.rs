use alloc::{
    boxed::Box,
    collections::BTreeMap,
    str,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{extract_zip, App, Archive, System};

use crate::app::J2MEApp;

pub struct J2MEArchive {
    files: BTreeMap<String, Vec<u8>>,
    manifest: J2MEManifest,
}

impl J2MEArchive {
    pub fn from_jar(data: Vec<u8>) -> Self {
        let files = extract_zip(&data).unwrap();

        let manifest_file = files.get("META-INF/MANIFEST.MF").unwrap();
        let manifest = J2MEManifest::parse(manifest_file);

        Self { files, manifest }
    }
}

impl Archive for J2MEArchive {
    fn id(&self) -> String {
        self.manifest.name.clone()
    }

    fn load_app(self: Box<Self>, system: System) -> anyhow::Result<Box<dyn App>> {
        let system_handle = system.handle();
        let mut resource = system_handle.resource_mut();

        for (path, data) in self.files {
            resource.add(&path, data);
        }

        Ok(Box::new(J2MEApp::new(&self.manifest.main_class_name, system)?))
    }
}

struct J2MEManifest {
    name: String,
    main_class_name: String,
}

impl J2MEManifest {
    pub fn parse(data: &[u8]) -> Self {
        let lines = data.split(|x| *x == b'\n');

        let mut name = String::new();
        let mut main_class_name = String::new();

        for line in lines {
            let line = str::from_utf8(line).unwrap().trim();

            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, ':');

            let key = parts.next().unwrap().trim();
            let value = parts.next().unwrap().trim();

            match key {
                "MIDlet-Name" => name = value.to_string(),
                "MIDlet-1" => main_class_name = value.split(',').nth(2).unwrap().trim().to_string(),
                _ => {}
            }
        }

        Self { name, main_class_name }
    }
}
