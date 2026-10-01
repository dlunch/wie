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

#[cfg(test)]
mod tests {
    use std::{panic, sync::mpsc, thread, time::Duration};

    use gtk::{
        cairo::{Context, Format, ImageSurface},
        prelude::*,
    };
    use tauri::Manager;

    use super::super::{Frame, NativeView, VIEW};

    fn on_ui<T: Send + 'static>(view: &NativeView, operation: impl FnOnce() -> T + Send + 'static) -> T {
        // Allow GTK's frame clock to apply queued allocation/draw changes.
        thread::sleep(Duration::from_millis(200));
        let (sender, receiver) = mpsc::sync_channel(1);
        view.dispatch(move || {
            sender.send(operation()).unwrap();
        })
        .unwrap();
        receiver.recv_timeout(Duration::from_secs(10)).unwrap()
    }

    fn rendered_pixels() -> (usize, usize, Vec<u32>) {
        VIEW.with_borrow(|view| {
            let view = view.as_ref().unwrap();
            let area = &view.native.area;
            assert!(area.is_visible());
            let (width, height) = (area.allocated_width(), area.allocated_height());
            assert!(width > 0 && height > 0);
            let mut image = ImageSurface::create(Format::Rgb24, width, height).unwrap();
            {
                let context = Context::new(&image).unwrap();
                area.draw(&context);
            }
            let pixels = image
                .data()
                .unwrap()
                .chunks_exact(4)
                .map(|pixel| u32::from_ne_bytes(pixel.try_into().unwrap()) & 0xffffff)
                .collect();
            (width as usize, height as usize, pixels)
        })
    }

    #[test]
    #[ignore = "requires an unoccluded desktop display or Xvfb, WebKitGTK, and an audio output"]
    fn native_view_presents_latest_frame_and_clears_on_restart() {
        let (sender, receiver) = mpsc::sync_channel(1);
        let app = tauri::Builder::default()
            .any_thread()
            .setup(move |app| {
                let window = app.get_webview_window("main").unwrap();
                let view = NativeView::new(window.clone()).unwrap();
                let library_size = window.inner_size().unwrap();
                let handle = app.handle().clone();
                let worker = thread::spawn(move || {
                    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
                        view.set_playing(true).unwrap();
                        assert!(!window.is_resizable().unwrap());
                        view.present(Frame {
                            width: 1,
                            height: 1,
                            pixels: vec![0x00ff00],
                        })
                        .unwrap();
                        view.present(Frame {
                            width: 2,
                            height: 1,
                            pixels: vec![0xff0000, 0x0000ff],
                        })
                        .unwrap();
                        let (width, height, pixels) = on_ui(&view, rendered_pixels);
                        assert_eq!((width, height), (360, 480));
                        assert_eq!(
                            window.inner_size().unwrap().to_logical::<u32>(window.scale_factor().unwrap()),
                            tauri::LogicalSize::new(360, 740)
                        );
                        assert_eq!(pixels[(height / 2) * width + width / 4], 0xff0000);
                        assert_eq!(pixels[(height / 2) * width + width * 3 / 4], 0x0000ff);
                        assert_eq!(pixels[0], 0);

                        view.set_playing(false).unwrap();
                        let visible = on_ui(&view, || VIEW.with_borrow(|view| view.as_ref().unwrap().native.area.is_visible()));
                        assert!(window.is_resizable().unwrap());
                        assert_eq!(window.inner_size().unwrap(), library_size);
                        assert!(!visible);
                        view.present(Frame {
                            width: 1,
                            height: 1,
                            pixels: vec![0xff0000],
                        })
                        .unwrap();
                        view.set_playing(true).unwrap();
                        let (_, _, pixels) = on_ui(&view, rendered_pixels);
                        assert!(pixels.iter().all(|pixel| *pixel == 0));

                        view.set_playing(false).unwrap();
                        assert!(view.take_error().is_none());
                        tauri::async_runtime::block_on(crate::runtime::tests::check_session_lifecycle(handle.clone(), view));
                        window.destroy().unwrap();
                    }));
                    handle.exit(0);
                    result
                });
                sender.send(worker).unwrap();
                Ok(())
            })
            .build(tauri::generate_context!())
            .unwrap();
        assert_eq!(app.run_return(|_, _| {}), 0);
        if let Err(error) = receiver.recv().unwrap().join().unwrap() {
            panic::resume_unwind(error);
        }
    }
}
