use alloc::boxed::Box;

use crate::{audio_sink::AudioSink, database::DatabaseRepository, screen::Screen, time::Instant};

pub trait Platform: Send + Sync {
    fn screen(&self) -> &dyn Screen;
    fn now(&self) -> Instant;
    fn database_repository(&self) -> &dyn DatabaseRepository;
    fn audio_sink(&self) -> Box<dyn AudioSink>;
    fn write_stdout(&self, buf: &[u8]);
    fn write_stderr(&self, buf: &[u8]);
    fn exit(&self);
}
