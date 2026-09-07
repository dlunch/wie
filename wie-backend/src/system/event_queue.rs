use alloc::{boxed::Box, collections::VecDeque};
use core::pin::Pin;

use wie_util::Result;

use crate::Instant;

#[allow(clippy::upper_case_acronyms, non_camel_case_types)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyCode {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    OK,
    LEFT_SOFT_KEY,
    RIGHT_SOFT_KEY,
    CLEAR,
    CALL,
    HANGUP,
    VOLUME_UP,
    VOLUME_DOWN,

    NUM0,
    NUM1,
    NUM2,
    NUM3,
    NUM4,
    NUM5,
    NUM6,
    NUM7,
    NUM8,
    NUM9,
    HASH,
    STAR,
}

impl KeyCode {
    // TODO we can use libraries like strum
    pub fn parse(string: &str) -> KeyCode {
        match string {
            "UP" => KeyCode::UP,
            "DOWN" => KeyCode::DOWN,
            "LEFT" => KeyCode::LEFT,
            "RIGHT" => KeyCode::RIGHT,
            "OK" => KeyCode::OK,
            "0" => KeyCode::NUM0,
            "1" => KeyCode::NUM1,
            "2" => KeyCode::NUM2,
            "3" => KeyCode::NUM3,
            "4" => KeyCode::NUM4,
            "5" => KeyCode::NUM5,
            "6" => KeyCode::NUM6,
            "7" => KeyCode::NUM7,
            "8" => KeyCode::NUM8,
            "9" => KeyCode::NUM9,
            "#" => KeyCode::HASH,
            "*" => KeyCode::STAR,
            "CLR" => KeyCode::CLEAR,
            _ => unimplemented!("Unknown key: {string}"),
        }
    }
}

type TimerCallback = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = Result<()>> + Send>> + Send + Sync>;

pub enum Event {
    Redraw,
    Keydown(KeyCode),
    Keyup(KeyCode),
    Keyrepeat(KeyCode),
    Timer { due: Instant, callback: TimerCallback },
    Notify { r#type: i32, param1: i32, param2: i32 }, // wipi notifyEvent
}

impl Event {
    pub fn timer<F, Fut>(due: Instant, callback: F) -> Self
    where
        F: FnOnce() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<()>> + Send + 'static,
    {
        Event::Timer {
            due,
            callback: Box::new(move || Box::pin(callback())),
        }
    }
}

#[derive(Default)]
pub struct EventQueue {
    input_events: VecDeque<Event>,
    events: VecDeque<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, event: Event) {
        if matches!(event, Event::Keydown(_) | Event::Keyup(_) | Event::Keyrepeat(_)) {
            self.input_events.push_back(event);
            return;
        }
        if matches!(event, Event::Redraw) && self.events.iter().any(|event| matches!(event, Event::Redraw)) {
            return;
        }
        self.events.push_back(event);
    }

    /// Keyboard input takes priority; events at the same priority remain FIFO.
    pub fn pop(&mut self) -> Option<Event> {
        self.input_events.pop_front().or_else(|| self.events.pop_front())
    }
}

#[cfg(test)]
mod tests {
    use crate::Instant;

    use super::{Event, EventQueue, KeyCode};

    #[test]
    fn prioritizes_input_and_coalesces_pending_redraws() {
        let mut queue = EventQueue::new();
        queue.push(Event::Keydown(KeyCode::DOWN));
        queue.push(Event::Redraw);
        queue.push(Event::Keyrepeat(KeyCode::DOWN));
        for _ in 0..100 {
            queue.push(Event::Redraw);
        }
        queue.push(Event::Keyup(KeyCode::DOWN));

        assert!(matches!(queue.pop(), Some(Event::Keydown(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Keyrepeat(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Keyup(KeyCode::DOWN))));
        assert!(matches!(queue.pop(), Some(Event::Redraw)));
        // A repaint requested while painting still needs another delivery.
        queue.push(Event::Redraw);
        assert!(matches!(queue.pop(), Some(Event::Redraw)));
        assert!(queue.pop().is_none());
    }

    #[test]
    fn new_input_precedes_timers_and_notifications_without_reordering_them() {
        let mut queue = EventQueue::new();
        queue.push(Event::timer(Instant::from_epoch_millis(10), || async { Ok(()) }));
        queue.push(Event::Notify {
            r#type: 1,
            param1: 2,
            param2: 3,
        });
        queue.push(Event::Keydown(KeyCode::OK));
        assert!(matches!(queue.pop(), Some(Event::Keydown(KeyCode::OK))));
        queue.push(Event::Keyup(KeyCode::OK));
        assert!(matches!(queue.pop(), Some(Event::Keyup(KeyCode::OK))));
        assert!(matches!(queue.pop(), Some(Event::Timer { due, .. }) if due.raw() == 10));
        assert!(matches!(
            queue.pop(),
            Some(Event::Notify {
                r#type: 1,
                param1: 2,
                param2: 3
            })
        ));
        assert!(queue.pop().is_none());
    }
}
