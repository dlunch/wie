use crate::canvas::Canvas;

pub trait Screen {
    fn request_redraw(&self) -> anyhow::Result<()>;
    fn repaint(&self) -> anyhow::Result<()>;
    fn canvas(&mut self) -> &mut dyn Canvas;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}
