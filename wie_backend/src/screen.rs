use crate::canvas::Image;

use wie_util::Result;

pub trait Screen: Send {
    fn request_redraw(&self) -> Result<()>;
    fn paint(&mut self, image: &dyn Image);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
