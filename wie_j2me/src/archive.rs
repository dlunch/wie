use alloc::{
    boxed::Box,
    str,
    string::{String, ToString},
    vec::Vec,
};

use wie_backend::{App, Archive, Platform, System};

use crate::app::J2MEApp;

pub struct J2MEArchive {
    jar: Vec<u8>,
    name: String,
    main_class_name: Option<String>,
}

impl J2MEArchive {
    pub fn from_jad_jar(jad: Vec<u8>, jar: Vec<u8>) -> Self {
        let descriptor = J2MEDescriptor::parse(&jad);

        Self {
            jar,
            name: descriptor.name,
            main_class_name: Some(descriptor.main_class_name),
        }
    }

    pub fn from_jar(filename: String, jar: Vec<u8>) -> Self {
        Self {
            jar,
            name: filename,
            main_class_name: None,
        }
    }
}

impl Archive for J2MEArchive {
    fn id(&self) -> String {
        self.name.clone()
    }

    fn load_app(self: Box<Self>, platform: Box<dyn Platform>) -> anyhow::Result<Box<dyn App>> {
        let system = System::new(platform);

        Ok(Box::new(J2MEApp::new(self.main_class_name, self.jar, system)?))
    }
}

struct J2MEDescriptor {
    name: String,
    main_class_name: String,
}

impl J2MEDescriptor {
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
