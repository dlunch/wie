mod audio;
mod event_queue;
mod random;
mod resource;

use alloc::rc::Rc;
use core::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
};

use crate::{
    executor::Executor,
    platform::Platform,
    task::{SleepFuture, YieldFuture},
    AsyncCallable, Instant,
};

use self::{audio::Audio, event_queue::EventQueue, random::Random, resource::Resource};

#[derive(Clone)]
pub struct System {
    executor: Executor,
    platform: Rc<RefCell<Box<dyn Platform>>>,
    resource: Rc<RefCell<Resource>>,
    event_queue: Rc<RefCell<EventQueue>>,
    audio: Rc<RefCell<Audio>>,
    random: Rc<RefCell<Random>>,
    context: Rc<RefCell<Box<dyn Any>>>,
}

impl System {
    pub fn new(platform: Box<dyn Platform>, context: Box<dyn Any>) -> Self {
        let audio_sink = platform.audio_sink();
        let seed = 12341234; // TODO get seed from outside

        Self {
            executor: Executor::new(),
            platform: Rc::new(RefCell::new(platform)),
            resource: Rc::new(RefCell::new(Resource::new())),
            event_queue: Rc::new(RefCell::new(EventQueue::new())),
            audio: Rc::new(RefCell::new(Audio::new(audio_sink))),
            random: Rc::new(RefCell::new(Random::new(seed))),
            context: Rc::new(RefCell::new(context)),
        }
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        let platform = self.platform.clone();
        self.executor.tick(move || {
            let platform = platform.borrow();

            platform.now()
        })
    }

    pub fn spawn<C, R, E>(&mut self, callable: C)
    where
        C: AsyncCallable<R, E> + 'static,
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

    pub fn resource(&self) -> Ref<'_, Resource> {
        self.resource.borrow()
    }

    pub fn resource_mut(&self) -> RefMut<'_, Resource> {
        self.resource.borrow_mut()
    }

    pub fn platform(&self) -> RefMut<'_, Box<dyn Platform>> {
        self.platform.borrow_mut()
    }

    pub fn audio(&self) -> RefMut<'_, Audio> {
        self.audio.borrow_mut()
    }

    pub fn event_queue(&self) -> RefMut<'_, EventQueue> {
        self.event_queue.borrow_mut()
    }

    pub fn random(&self) -> RefMut<'_, Random> {
        self.random.borrow_mut()
    }

    pub fn context(&self) -> RefMut<'_, Box<dyn Any>> {
        self.context.borrow_mut()
    }
}
