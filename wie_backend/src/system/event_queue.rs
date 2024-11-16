use alloc::collections::VecDeque;

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum KeyCode {
    UP,
    DOWN,
    LEFT,
    RIGHT,
    OK,

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
    CLEAR,
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
            _ => unimplemented!("Unknown key: {}", string),
        }
    }
}

#[derive(Debug)]
pub enum Event {
    Redraw,
    Keydown(KeyCode),
    Keyup(KeyCode),
    Keyrepeat(KeyCode),
}

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
