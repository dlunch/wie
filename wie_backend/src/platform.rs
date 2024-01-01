use crate::{system::time::Instant, Window};

pub trait Platform {
    fn window(&self) -> &dyn Window;
    fn now(&self) -> Instant;
}
