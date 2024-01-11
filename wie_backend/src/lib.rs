extern crate alloc;

mod audio_sink;
pub mod canvas;
mod database;
mod executor;
mod platform;
mod screen;
mod system;
mod task;
mod time;

pub use self::{
    audio_sink::AudioSink,
    database::{Database, DatabaseRepository, RecordId},
    executor::AsyncCallable,
    platform::Platform,
    screen::Screen,
    system::{System, SystemHandle},
    time::Instant,
};

use alloc::{boxed::Box, collections::BTreeMap, string::String};

pub trait App {
    fn start(&mut self) -> anyhow::Result<()>;
    fn on_event(&mut self, event: wie_common::Event);
    fn tick(&mut self) -> anyhow::Result<()>;
    fn crash_dump(&self) -> String;
}

pub trait Archive {
    fn id(&self) -> String;
    fn load_app(self: Box<Self>, system: System) -> anyhow::Result<Box<dyn App>>;
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
