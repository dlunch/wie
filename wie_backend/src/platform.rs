use crate::Window;

pub trait Platform {
    fn create_window(&self) -> Box<dyn Window>;
}
