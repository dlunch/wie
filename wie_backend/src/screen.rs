use crate::canvas::Image;

pub trait Screen {
    fn request_redraw(&self) -> anyhow::Result<()>;
    fn paint(&mut self, image: &dyn Image);
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
