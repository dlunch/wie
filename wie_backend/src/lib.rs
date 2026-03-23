#![no_std]
extern crate alloc;

mod audio_sink;
pub mod canvas;
mod database;
mod executor;
mod platform;
mod screen;
mod system;
mod task;
mod task_runner;
mod time;

pub use self::{
    audio_sink::AudioSink,
    database::{Database, DatabaseRepository, RecordId},
    executor::{AsyncCallable, AsyncCallableResult},
    platform::Platform,
    screen::Screen,
    system::{Event, KeyCode, System},
    task_runner::{DefaultTaskRunner, TaskRunner},
    time::Instant,
};

use alloc::{
    collections::BTreeMap,
    format,
    string::{String, ToString},
    vec::Vec,
};

use wie_util::{Result, WieError};

pub trait Emulator {
    fn handle_event(&mut self, event: Event);
    fn tick(&mut self) -> Result<()>;
}

pub struct Options {
    pub enable_gdbserver: bool,
}

pub fn extract_zip(zip: &[u8]) -> Result<BTreeMap<String, Vec<u8>>> {
    extern crate std; // XXX

    use std::io::{Cursor, Read};
    use zip::ZipArchive;

    let mut archive = ZipArchive::new(Cursor::new(zip)).map_err(|x| WieError::FatalError(format!("Invalid zip archive: {x}")))?;

    (0..archive.len())
        .filter_map(|x| {
            let mut file = match archive.by_index(x) {
                Ok(file) => file,
                Err(err) => return Some(Err(WieError::FatalError(format!("Failed to read zip entry {x}: {err}")))),
            };
            if !file.is_file() {
                return None;
            }

            let mut data = Vec::new();
            if let Err(err) = file.read_to_end(&mut data) {
                return Some(Err(WieError::FatalError(format!("Failed to read zip entry {}: {err}", file.name()))));
            }

            Some(Ok((file.name().to_string(), data)))
        })
        .collect::<Result<_>>()
}
