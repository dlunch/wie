extern crate alloc;

mod executor;
mod platform;
mod system;
pub mod task;

pub use self::{
    executor::{AsyncCallable, Executor},
    platform::Platform,
    system::{canvas, database::Database, window::Window, System},
};

use alloc::{boxed::Box, collections::BTreeMap, string::String};

pub trait App {
    fn start(&mut self) -> anyhow::Result<()>;
    fn crash_dump(&self) -> String;
}

pub trait Archive {
    fn id(&self) -> String;
    fn load_app(&self, system: &mut System) -> anyhow::Result<Box<dyn App>>;
}

pub fn extract_zip(zip: &[u8]) -> anyhow::Result<BTreeMap<String, Vec<u8>>> {
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let mut archive = ZipArchive::new(Cursor::new(zip))?;

    (0..archive.len())
        .map(|x| {
            let mut file = archive.by_index(x)?;

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            Ok((file.name().to_string(), data))
        })
        .collect::<anyhow::Result<_>>()
}

// assume system encoding is euc-kr
pub fn encode_str(string: &str) -> Vec<u8> {
    use encoding_rs::EUC_KR;

    EUC_KR.encode(string).0.to_vec()
}

pub fn decode_str(bytes: &[u8]) -> String {
    use encoding_rs::EUC_KR;

    EUC_KR.decode(bytes).0.to_string()
}
