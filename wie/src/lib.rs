#![no_std]

extern crate alloc;

use alloc::boxed::Box;

use wie_backend::Backend;
use wie_base::App;
use wie_vendor_ktf::{is_ktf_archive_loaded, load_ktf_archive};

fn load_archive(file: &[u8], backend: &mut Backend) -> anyhow::Result<Box<dyn App>> {
    backend.add_resources_from_zip(file)?;

    if is_ktf_archive_loaded(backend) {
        return load_ktf_archive(backend);
    }

    anyhow::bail!("Unknown vendor")
}

pub fn start(file: &[u8]) -> anyhow::Result<()> {
    let mut backend = Backend::new();

    let app = load_archive(file, &mut backend)?;

    backend.run(app)?;

    Ok(())
}
