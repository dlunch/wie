use crate::canvas::Canvas;

pub trait Window {
    fn request_redraw(&self) -> anyhow::Result<()>;

    fn repaint(&self, canvas: &dyn Canvas) -> anyhow::Result<()>;
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}
