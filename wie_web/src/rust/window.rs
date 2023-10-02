use alloc::{rc::Rc, vec::Vec};
use core::cell::Cell;

use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

use wie_backend::{canvas::Canvas, Window};

pub struct WindowImpl {
    canvas: HtmlCanvasElement,
    should_redraw: Rc<Cell<bool>>,
}

impl WindowImpl {
    pub fn new(canvas: HtmlCanvasElement, should_redraw: Rc<Cell<bool>>) -> Self {
        Self { canvas, should_redraw }
    }
}

impl Window for WindowImpl {
    fn request_redraw(&self) -> anyhow::Result<()> {
        self.should_redraw.set(true);

        Ok(())
    }

    fn repaint(&self, canvas: &dyn Canvas) -> anyhow::Result<()> {
        let context = self
            .canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .unwrap();

        let image_data = canvas.colors().into_iter().flat_map(|x| [x.r, x.g, x.b, x.a]).collect::<Vec<_>>();
        let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&image_data), canvas.width(), canvas.height()).unwrap();

        context.put_image_data(&data, 0.0, 0.0).unwrap();

        Ok(())
    }

    fn width(&self) -> u32 {
        self.canvas.width()
    }

    fn height(&self) -> u32 {
        self.canvas.height()
    }
}
