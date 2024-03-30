mod audio;
mod event_queue;
mod resource;

use alloc::sync::Arc;
use core::{any::Any, fmt::Debug};
use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::{
    executor::Executor,
    platform::Platform,
    task::{SleepFuture, YieldFuture},
    AsyncCallable, Instant,
};

use self::{audio::Audio, event_queue::EventQueue, resource::Resource};

pub use self::event_queue::{Event, KeyCode};

#[derive(Clone)]
pub struct System {
    executor: Executor,
    platform: Arc<Mutex<Box<dyn Platform>>>,
    resource: Arc<RwLock<Resource>>,
    event_queue: Arc<RwLock<EventQueue>>,
    audio: Option<Arc<RwLock<Audio>>>,
    context: Arc<RwLock<Box<dyn Any + Sync + Send>>>,
}

impl System {
    pub fn new(platform: Box<dyn Platform>, context: Box<dyn Any + Sync + Send>) -> Self {
        let audio_sink = platform.audio_sink();

        let platform = Arc::new(Mutex::new(platform));

        let mut result = Self {
            executor: Executor::new(),
            platform: platform.clone(),
            resource: Arc::new(RwLock::new(Resource::new())),
            event_queue: Arc::new(RwLock::new(EventQueue::new())),
            audio: None,
            context: Arc::new(RwLock::new(context)),
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

    pub fn spawn<C, R, E>(&mut self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static + Send,
        E: Debug,
    {
        self.executor.spawn(callable);
    }

    pub fn sleep(&mut self, until: Instant) -> SleepFuture {
        SleepFuture::new(until, &mut self.executor)
    }

    pub fn yield_now(&self) -> YieldFuture {
        YieldFuture {}
    }

    // TODO add encoding configuration..
    pub fn encode_str(&self, string: &str) -> Vec<u8> {
        use encoding_rs::EUC_KR;

        EUC_KR.encode(string).0.to_vec()
    }

    pub fn decode_str(&self, bytes: &[u8]) -> String {
        use encoding_rs::EUC_KR;

        EUC_KR.decode(bytes).0.to_string()
    }

    pub fn resource(&self) -> RwLockReadGuard<'_, Resource> {
        self.resource.read().unwrap()
    }

    pub fn resource_mut(&self) -> RwLockWriteGuard<'_, Resource> {
        self.resource.write().unwrap()
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
    pub fn context(&self) -> RwLockWriteGuard<'_, Box<dyn Any + Sync + Send>> {
        self.context.write().unwrap()
    }
}
