use alloc::collections::VecDeque;

use wie_base::Event;

#[derive(Default)]
pub struct EventQueue {
    events: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self { events: VecDeque::new() }
    }

    pub fn push(&mut self, event: Event) {
        self.events.push_back(event);
    }

    pub fn pop(&mut self) -> Option<Event> {
        self.events.pop_front()
    }
}
