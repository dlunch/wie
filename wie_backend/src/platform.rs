use crate::{database::DatabaseRepository, screen::Screen, time::Instant};

pub trait Platform {
    fn screen(&mut self) -> &mut dyn Screen;
    fn now(&self) -> Instant;
    fn database_repository(&self) -> &dyn DatabaseRepository;
}
