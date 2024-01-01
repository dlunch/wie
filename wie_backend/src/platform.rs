use crate::{time::Instant, Screen};

pub trait Platform {
    fn screen(&mut self) -> &mut dyn Screen;
    fn now(&self) -> Instant;
}
