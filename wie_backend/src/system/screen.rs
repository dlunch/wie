use crate::canvas::Canvas;

pub trait Screen {
    fn request_redraw(&self) -> anyhow::Result<()>;

    fn repaint(&self, canvas: &dyn Canvas) -> anyhow::Result<()>;
    fn width(&self) -> u32;

    fn height(&self) -> u32;
}
