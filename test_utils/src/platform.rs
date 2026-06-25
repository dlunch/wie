use alloc::{
    boxed::Box,
    string::{String, ToString},
    sync::Arc,
    vec::Vec,
};
use core::sync::atomic::{AtomicU64, Ordering};

use hashbrown::HashMap;
use spin::Mutex;
use wie_backend::{AudioSink, Database, DatabaseRepository, Filesystem, Instant, Platform, RecordId, Screen, canvas::Image};
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
    db: Arc<MemoryDatabaseRepository>,
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
            db: Arc::new(MemoryDatabaseRepository::default()),
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
            db: Arc::new(MemoryDatabaseRepository::default()),
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
        self.db.as_ref()
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

type DatabaseKey = (String, String);
type DatabaseStore = HashMap<DatabaseKey, HashMap<RecordId, Vec<u8>>>;

#[derive(Default)]
struct MemoryDatabaseRepository {
    store: Arc<Mutex<DatabaseStore>>,
}

#[async_trait::async_trait]
impl DatabaseRepository for MemoryDatabaseRepository {
    async fn open(&self, name: &str, app_id: &str) -> Box<dyn Database> {
        let key = (app_id.to_string(), name.to_string());
        self.store.lock().entry(key.clone()).or_default();
        Box::new(MemoryDatabase {
            store: self.store.clone(),
            key,
        })
    }

    async fn exists(&self, name: &str, app_id: &str) -> bool {
        self.store.lock().contains_key(&(app_id.to_string(), name.to_string()))
    }

    async fn delete(&self, name: &str, app_id: &str) -> bool {
        self.store.lock().remove(&(app_id.to_string(), name.to_string())).is_some()
    }
}

struct MemoryDatabase {
    store: Arc<Mutex<DatabaseStore>>,
    key: DatabaseKey,
}

#[async_trait::async_trait]
impl Database for MemoryDatabase {
    async fn next_id(&self) -> RecordId {
        let store = self.store.lock();
        let records = store.get(&self.key);
        let mut id = 1;
        while records.is_some_and(|records| records.contains_key(&id)) {
            id += 1;
        }
        id
    }

    async fn add(&mut self, data: &[u8]) -> RecordId {
        let id = self.next_id().await;
        self.set(id, data).await;
        id
    }

    async fn get(&self, id: RecordId) -> Option<Vec<u8>> {
        self.store.lock().get(&self.key)?.get(&id).cloned()
    }

    async fn set(&mut self, id: RecordId, data: &[u8]) -> bool {
        let mut store = self.store.lock();
        store.entry(self.key.clone()).or_default().insert(id, data.to_vec());
        true
    }

    async fn delete(&mut self, id: RecordId) -> bool {
        self.store.lock().get_mut(&self.key).is_some_and(|records| records.remove(&id).is_some())
    }

    async fn get_record_ids(&self) -> Vec<RecordId> {
        self.store
            .lock()
            .get(&self.key)
            .map(|records| records.keys().copied().collect())
            .unwrap_or_default()
    }
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

    fn midi_pitch_bend(&self, _channel_id: u8, _value: u16) {
        todo!()
    }

    fn midi_sysex(&self, _data: &[u8]) {
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
