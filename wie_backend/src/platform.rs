use crate::{database::DatabaseRepository, time::Instant, Screen};

pub trait Platform {
    fn screen(&mut self) -> &mut dyn Screen;
    fn now(&self) -> Instant;
    fn database_repository(&self) -> &dyn DatabaseRepository;
}
