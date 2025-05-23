mod audio;
mod event_queue;
mod file_system;

use alloc::{borrow::ToOwned, boxed::Box, string::String, sync::Arc};

use spin::{Mutex, MutexGuard, RwLock, RwLockWriteGuard};

use wie_util::Result;

use crate::{
    AsyncCallable, Instant,
    executor::Executor,
    platform::Platform,
    task::{SleepFuture, YieldFuture},
    task_runner::TaskRunner,
};

use self::{audio::Audio, event_queue::EventQueue, file_system::Filesystem};

pub use self::event_queue::{Event, KeyCode};

#[derive(Clone)]
pub struct System {
    app_id: String,
    executor: Executor,
    platform: Arc<Mutex<Box<dyn Platform>>>,
    filesystem: Arc<Mutex<Filesystem>>,
    event_queue: Arc<RwLock<EventQueue>>,
    audio: Option<Arc<RwLock<Audio>>>,
    task_runner: Arc<dyn TaskRunner>,
}

impl System {
    pub fn new<T>(platform: Box<dyn Platform>, app_id: &str, task_runner: T) -> Self
    where
        T: TaskRunner + 'static,
    {
        let audio_sink = platform.audio_sink();

        let platform = Arc::new(Mutex::new(platform));

        let mut result = Self {
            app_id: app_id.to_owned(),
            executor: Executor::new(),
            platform: platform.clone(),
            filesystem: Arc::new(Mutex::new(Filesystem::new())),
            event_queue: Arc::new(RwLock::new(EventQueue::new())),
            audio: None,
            task_runner: Arc::new(task_runner),
        };

        // late initialization
        result.audio = Some(Arc::new(RwLock::new(Audio::new(audio_sink, result.clone()))));

        result
    }

    pub fn tick(&mut self) -> Result<()> {
        let platform = self.platform.clone();
        self.executor.tick(move || {
            let platform = platform.lock();

            platform.now()
        })
    }

    pub fn spawn<C>(&mut self, callable: C)
    where
        C: AsyncCallable<Result<()>> + 'static + Send,
    {
        let runner_clone = self.task_runner.clone();
        self.executor.spawn(async move || runner_clone.run(Box::pin(callable.call())).await);
    }

    pub fn sleep(&mut self, until: Instant) -> SleepFuture {
        SleepFuture::new(until, &mut self.executor)
    }

    pub fn current_task_id(&self) -> u64 {
        self.executor.current_task_id()
    }

    pub fn yield_now(&self) -> YieldFuture {
        YieldFuture::new()
    }

    pub fn filesystem(&self) -> MutexGuard<'_, Filesystem> {
        self.filesystem.lock()
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn platform(&self) -> MutexGuard<'_, Box<dyn Platform>> {
        self.platform.lock()
    }

    pub fn audio(&self) -> RwLockWriteGuard<'_, Audio> {
        self.audio.as_ref().unwrap().write()
    }

    pub fn event_queue(&self) -> RwLockWriteGuard<'_, EventQueue> {
        self.event_queue.write()
    }
}
