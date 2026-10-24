use alloc::sync::Arc;
use std::sync::Mutex;

use wie_backend::{Screen, canvas::Image};
use wie_renderer::Frame;
use wie_util::Result;

pub struct Pending {
    pub width: u32,
    pub height: u32,
    pub redraw: bool,
    pub frame: Option<Frame>,
}

pub struct WindowImpl {
    pub pending: Arc<Mutex<Pending>>,
}

impl Screen for WindowImpl {
    fn resize(&self, width: u32, height: u32) -> Result<()> {
        let mut pending = self.pending.lock().unwrap();
        pending.width = width;
        pending.height = height;
        pending.redraw = true;
        Ok(())
    }

    fn request_redraw(&self) -> Result<()> {
        self.pending.lock().unwrap().redraw = true;
        Ok(())
    }

    fn paint(&self, image: &dyn Image) {
        self.pending.lock().unwrap().frame = Some(Frame::from_image(image));
    }

    fn width(&self) -> u32 {
        self.pending.lock().unwrap().width
    }

    fn height(&self) -> u32 {
        self.pending.lock().unwrap().height
    }
}
