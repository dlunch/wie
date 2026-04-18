use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicU64, Ordering};

use wie_backend::{AudioSink, DatabaseRepository, Filesystem, Instant, Platform, Screen, canvas::Image};
use wie_util::Result;

use crate::filesystem::MemoryFilesystem;

static TEST_EPOCH: AtomicU64 = AtomicU64::new(0);

pub enum TestPlatformEvent {
    Stdout(Vec<u8>),
    Exit,
}

pub struct TestPlatform {
    screen: TestScreen,
    event_handler: Option<Box<dyn Fn(TestPlatformEvent) + Sync + Send>>,
    fs: Arc<MemoryFilesystem>,
}

impl Default for TestPlatform {
    fn default() -> Self {
        Self::new()
    }
}

impl TestPlatform {
    pub fn new() -> Self {
        Self {
            screen: TestScreen,
            event_handler: None,
            fs: Arc::new(MemoryFilesystem::default()),
        }
    }

    pub fn with_event_handler<T>(event_handler: T) -> Self
    where
        T: Fn(TestPlatformEvent) + Sync + Send + 'static,
    {
        Self {
            screen: TestScreen,
            event_handler: Some(Box::new(event_handler)),
            fs: Arc::new(MemoryFilesystem::default()),
        }
    }
}

impl Platform for TestPlatform {
    fn screen(&self) -> &dyn Screen {
        &self.screen
    }

    fn now(&self) -> Instant {
        let epoch = TEST_EPOCH.fetch_add(8, Ordering::SeqCst);
        Instant::from_epoch_millis(epoch) // TODO
    }

    fn database_repository(&self) -> &dyn DatabaseRepository {
        todo!()
    }

    fn filesystem(&self) -> &dyn Filesystem {
        self.fs.as_ref()
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(TestAudioSink)
    }

    fn write_stdout(&self, buf: &[u8]) {
        if let Some(event_handler) = &self.event_handler {
            (event_handler)(TestPlatformEvent::Stdout(buf.to_vec()))
        }
    }

    fn write_stderr(&self, _buf: &[u8]) {}

    fn exit(&self) {
        if let Some(event_handler) = &self.event_handler {
            (event_handler)(TestPlatformEvent::Exit);
        }
    }

    fn vibrate(&self, _duration_ms: u64, _intensity: u8) {}
}

pub struct TestAudioSink;

impl AudioSink for TestAudioSink {
    fn play_wave(&self, _channel: u8, _sampling_rate: u32, _wave_data: &[i16]) {
        todo!()
    }

    fn midi_note_on(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_note_off(&self, _channel_id: u8, _note: u8, _velocity: u8) {
        todo!()
    }

    fn midi_program_change(&self, _channel_id: u8, _program: u8) {
        todo!()
    }

    fn midi_control_change(&self, _channel_id: u8, _control: u8, _value: u8) {
        todo!()
    }
}

#[derive(Default)]
pub struct TestScreen;

impl Screen for TestScreen {
    fn request_redraw(&self) -> Result<()> {
        Ok(())
    }

    fn paint(&self, _image: &dyn Image) {}

    fn width(&self) -> u32 {
        320
    }

    fn height(&self) -> u32 {
        240
    }
}
