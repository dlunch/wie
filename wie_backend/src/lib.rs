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
    executor::{AsyncCallable, AsyncCallableResult},
    platform::Platform,
    screen::Screen,
    system::{Event, KeyCode, System},
    time::Instant,
};

use alloc::collections::BTreeMap;

use wie_util::Result;

pub trait Emulator {
    fn handle_event(&mut self, event: Event);
    fn tick(&mut self) -> Result<()>;
}

pub fn extract_zip(zip: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let mut archive = ZipArchive::new(Cursor::new(zip)).unwrap();

    (0..archive.len())
        .map(|x| {
            let mut file = archive.by_index(x).unwrap();

            let mut data = Vec::new();
            file.read_to_end(&mut data).unwrap();

            Ok((file.name().to_string(), data))
        })
        .collect::<Result<_>>()
}
