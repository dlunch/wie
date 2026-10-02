use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use anyhow::{Result, anyhow};
use gtk::{
    DrawingArea,
    cairo::{Filter, Format, ImageSurface},
    glib::Propagation,
    prelude::*,
};
use tauri::webview::PlatformWebview;

use super::{CONTROLS_HEIGHT, Frame, GAME_HEIGHT, GAME_WIDTH, report_error};

pub(super) struct View {
    area: DrawingArea,
    image: RefCell<Option<ImageSurface>>,
    window: gtk::Window,
    library_default_size: Cell<Option<(i32, i32)>>,
}

impl View {
    pub(super) fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let webview = webview.inner();
        let parent = webview.parent().ok_or_else(|| anyhow!("WebView has no GTK parent"))?;
        let container = parent.downcast::<gtk::Box>().map_err(|_| anyhow!("WebView parent is not a GtkBox"))?;
        let window = container
            .toplevel()
            .and_downcast::<gtk::Window>()
            .ok_or_else(|| anyhow!("GTK window is unavailable"))?;
        let area = DrawingArea::new();
        area.set_size_request(-1, GAME_HEIGHT as i32);
        area.set_hexpand(true);
        area.set_no_show_all(true);
        container.pack_start(&area, false, false, 0);
        container.reorder_child(&area, 0);
        let view = Rc::new(Self {
            area,
            image: RefCell::new(None),
            window,
            library_default_size: Cell::new(None),
        });
        let weak = Rc::downgrade(&view);
        view.area.connect_draw(move |area, context| {
            let result = (|| -> Result<()> {
                context.set_source_rgb(0.0, 0.0, 0.0);
                context.paint()?;
                if let Some(view) = weak.upgrade()
                    && let Some(image) = view.image.borrow().as_ref()
                {
                    let width = f64::from(area.allocated_width());
                    let height = f64::from(area.allocated_height());
                    let scale = (width / f64::from(image.width())).min(height / f64::from(image.height()));
                    context.save()?;
                    context.translate(
                        (width - f64::from(image.width()) * scale) / 2.0,
                        (height - f64::from(image.height()) * scale) / 2.0,
                    );
                    context.scale(scale, scale);
                    context.set_source_surface(image, 0.0, 0.0)?;
                    context.source().set_filter(Filter::Nearest);
                    context.paint()?;
                    context.restore()?;
                }
                Ok(())
            })();
            if let Err(error) = result {
                report_error(error);
            }
            Propagation::Stop
        });
        Ok(view)
    }

    pub(super) fn set_playing(&self, playing: bool) -> Result<()> {
        if playing {
            if self.library_default_size.get().is_none() {
                self.library_default_size.set(Some(self.window.default_size()));
            }
            // GTK uses the default size as the minimum while the window is non-resizable.
            self.window.set_default_size(GAME_WIDTH as i32, (GAME_HEIGHT + CONTROLS_HEIGHT) as i32);
            self.area.show();
        } else {
            self.area.hide();
            self.image.take();
            if let Some((width, height)) = self.library_default_size.take() {
                self.window.set_default_size(width, height);
            }
        }
        Ok(())
    }

    pub(super) fn present(&self, frame: Frame) -> Result<()> {
        let data: Vec<u8> = frame.pixels.into_iter().flat_map(u32::to_ne_bytes).collect();
        let image = ImageSurface::create_for_data(data, Format::Rgb24, frame.width as i32, frame.height as i32, frame.width as i32 * 4)?;
        self.image.replace(Some(image));
        self.area.queue_draw();
        Ok(())
    }

    pub(super) fn resize(&self) -> Result<()> {
        self.area.queue_draw();
        Ok(())
    }

    pub(super) fn close(&self) {
        self.image.take();
        if let Some(parent) = self.area.parent().and_downcast::<gtk::Container>() {
            parent.remove(&self.area);
        }
    }
}
