mod audio;
mod event_queue;
mod file_system;

use alloc::sync::Arc;
use std::sync::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};

use crate::{
    executor::{AsyncCallableResult, Executor},
    platform::Platform,
    task::{SleepFuture, YieldFuture},
    AsyncCallable, Instant,
};

use self::{audio::Audio, event_queue::EventQueue, file_system::Filesystem};

pub use self::event_queue::{Event, KeyCode};

#[derive(Clone)]
pub struct System {
    executor: Executor,
    platform: Arc<Mutex<Box<dyn Platform>>>,
    filesystem: Arc<Mutex<Filesystem>>,
    event_queue: Arc<RwLock<EventQueue>>,
    audio: Option<Arc<RwLock<Audio>>>,
}

impl System {
    pub fn new(platform: Box<dyn Platform>) -> Self {
        let audio_sink = platform.audio_sink();

        let platform = Arc::new(Mutex::new(platform));

        let mut result = Self {
            executor: Executor::new(),
            platform: platform.clone(),
            filesystem: Arc::new(Mutex::new(Filesystem::new())),
            event_queue: Arc::new(RwLock::new(EventQueue::new())),
            audio: None,
        };

        // late initialization
        result.audio = Some(Arc::new(RwLock::new(Audio::new(audio_sink, result.clone()))));

        result
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        let platform = self.platform.clone();
        self.executor.tick(move || {
            let platform = platform.lock().unwrap();

            platform.now()
        })
    }

    pub fn spawn<C, R>(&mut self, callable: C)
    where
        C: AsyncCallable<R> + 'static + Send,
        R: AsyncCallableResult,
    {
        self.executor.spawn(callable);
    }

    pub fn sleep(&mut self, until: Instant) -> SleepFuture {
        SleepFuture::new(until, &mut self.executor)
    }

    pub fn current_task_id(&self) -> u64 {
        self.executor.current_task_id()
    }

    pub fn yield_now(&self) -> YieldFuture {
        YieldFuture {}
    }

    pub fn filesystem(&self) -> MutexGuard<'_, Filesystem> {
        self.filesystem.lock().unwrap()
    }

    pub fn platform(&self) -> MutexGuard<'_, Box<dyn Platform>> {
        self.platform.lock().unwrap()
    }

    pub fn audio(&self) -> RwLockWriteGuard<'_, Audio> {
        self.audio.as_ref().unwrap().write().unwrap()
    }

    pub fn event_queue(&self) -> RwLockWriteGuard<'_, EventQueue> {
        self.event_queue.write().unwrap()
    }
}
