use alloc::{boxed::Box, vec::Vec};

use wie_backend::{extract_zip, App, Archive, Backend};

use crate::app::J2MEApp;

pub struct J2MEArchive {
    jar: Vec<u8>,
}

impl J2MEArchive {
    pub fn from_jar(data: Vec<u8>) -> Self {
        Self { jar: data } // TODO get main class from manifest
    }
}

impl Archive for J2MEArchive {
    fn load_app(&self, backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
        let jar_data = extract_zip(&self.jar)?;

        for (filename, data) in jar_data {
            backend.add_resource(&filename, data);
        }

        Ok(Box::new(J2MEApp::new("", backend)?))
    }
}
