use crate::{system::time::Instant, Screen};

pub trait Platform {
    fn screen(&self) -> &dyn Screen;
    fn now(&self) -> Instant;
}
