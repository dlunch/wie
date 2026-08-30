use crate::canvas::Image;

use wie_util::Result;

pub trait Screen: Send + Sync {
    fn resize(&self, width: u32, height: u32) -> Result<()>;
    fn request_redraw(&self) -> Result<()>;
    fn paint(&self, image: &dyn Image);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
