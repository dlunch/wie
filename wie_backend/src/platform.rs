use crate::{audio_sink::AudioSink, database::DatabaseRepository, screen::Screen, time::Instant};

pub trait Platform: Send {
    fn screen(&mut self) -> &mut dyn Screen;
    fn now(&self) -> Instant;
    fn database_repository(&self) -> &dyn DatabaseRepository;
    fn audio_sink(&self) -> Box<dyn AudioSink>;
}
