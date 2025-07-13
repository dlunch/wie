use std::{
    sync::Arc,
    sync::{
        Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
};

use test_utils::TestAudioSink;
use wie_backend::{AudioSink, Emulator, Instant, Options, Platform, Screen, canvas::Image, extract_zip};
use wie_ktf::KtfEmulator;
use wie_util::Result;

struct TestScreen;

impl Screen for TestScreen {
    fn request_redraw(&self) -> Result<()> {
        Ok(())
    }

    fn paint(&mut self, _image: &dyn Image) {}

    fn width(&self) -> u32 {
        320
    }

    fn height(&self) -> u32 {
        240
    }
}

static TEST_EPOCH: AtomicU64 = AtomicU64::new(0);
struct TestPlatform {
    stdout: Arc<Mutex<Vec<u8>>>,
    exited: Arc<AtomicBool>,
    screen: TestScreen,
}

impl TestPlatform {
    pub fn new(stdout: Arc<Mutex<Vec<u8>>>, exited: Arc<AtomicBool>) -> Self {
        Self {
            stdout,
            exited,
            screen: TestScreen,
        }
    }
}

impl Platform for TestPlatform {
    fn screen(&mut self) -> &mut dyn Screen {
        &mut self.screen
    }

    fn now(&self) -> Instant {
        let epoch = TEST_EPOCH.fetch_add(8, Ordering::SeqCst);
        Instant::from_epoch_millis(epoch) // tODO
    }

    fn database_repository(&self) -> &dyn wie_backend::DatabaseRepository {
        todo!()
    }

    fn audio_sink(&self) -> Box<dyn AudioSink> {
        Box::new(TestAudioSink)
    }

    fn write_stdout(&self, buf: &[u8]) {
        let mut stdout = self.stdout.lock().unwrap();
        stdout.extend_from_slice(buf);
    }

    fn write_stderr(&self, _buf: &[u8]) {}

    fn exit(&self) {
        self.exited.store(true, Ordering::SeqCst);
    }
}

#[test]
pub fn test_helloworld() -> Result<()> {
    let stdout = Arc::new(Mutex::new(Vec::new()));
    let exited = Arc::new(AtomicBool::new(false));
    let platform = Box::new(TestPlatform::new(stdout.clone(), exited.clone()));

    let archive = extract_zip(include_bytes!("../../test_data/helloworld.zip"))?;
    let mut emulator = KtfEmulator::from_archive(platform, archive, Options { enable_gdbserver: false })?;

    while !exited.load(Ordering::SeqCst) {
        emulator.tick()?;
    }

    let stdout_str = String::from_utf8(stdout.lock().unwrap().clone()).unwrap();
    assert_eq!(stdout_str, "Hello, world!");

    Ok(())
}
