use crate::Window;

pub trait Platform {
    fn window(&self) -> &dyn Window;
}
