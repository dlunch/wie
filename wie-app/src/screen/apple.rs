use std::{
    cell::{Cell, RefCell},
    num::NonZeroU32,
    ptr::NonNull,
    rc::Rc,
};

use anyhow::{Result, anyhow};
use objc2::{MainThreadMarker, MainThreadOnly, rc::Retained};
use objc2_foundation::{NSPoint, NSRect, NSSize};
use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, WindowHandle};
use softbuffer::{Context, Surface};
use tauri::webview::PlatformWebview;

use objc2_app_kit::{NSAutoresizingMaskOptions, NSView as HostView};
use raw_window_handle::AppKitWindowHandle;

use super::{APP_HEIGHT, CONTROLS_HEIGHT, Frame};

struct AppView(Retained<HostView>);

impl HasWindowHandle for AppView {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        let pointer = NonNull::from(&*self.0).cast();
        let raw = AppKitWindowHandle::new(pointer);
        // The retained view outlives its surface and is accessed only on the main thread.
        Ok(unsafe { WindowHandle::borrow_raw(raw.into()) })
    }
}

pub(super) struct View {
    parent: Retained<HostView>,
    webview: Retained<HostView>,
    app: Rc<AppView>,
    surface: RefCell<Option<Surface<DisplayHandle<'static>, Rc<AppView>>>>,
    frame: RefCell<Option<Frame>>,
    playing: Cell<bool>,
}

impl View {
    pub(super) fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let main = MainThreadMarker::new().ok_or_else(|| anyhow!("Native view requires the main thread"))?;
        let webview = unsafe { Retained::<HostView>::retain(webview.inner().cast()) }.ok_or_else(|| anyhow!("WebView is unavailable"))?;
        let parent = unsafe { webview.superview() };
        let parent = parent.ok_or_else(|| anyhow!("WebView has no parent"))?;
        let app = Rc::new(AppView(HostView::initWithFrame(
            HostView::alloc(main),
            NSRect::new(NSPoint::ZERO, NSSize::ZERO),
        )));
        app.0.setHidden(true);

        webview.setAutoresizingMask(NSAutoresizingMaskOptions::empty());
        parent.addSubview(&app.0);
        Ok(Rc::new(Self {
            parent,
            webview,
            app,
            surface: RefCell::new(None),
            frame: RefCell::new(None),
            playing: Cell::new(false),
        }))
    }

    pub(super) fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if playing && self.surface.borrow().is_none() {
            let display = DisplayHandle::appkit();
            let context = Context::new(display).map_err(|error| anyhow!(error.to_string()))?;
            let surface = Surface::new(&context, self.app.clone()).map_err(|error| anyhow!(error.to_string()))?;
            self.surface.replace(Some(surface));
        } else if !playing {
            self.frame.take();
            self.surface.take();
            // Softbuffer removes its observers, but its sublayer remains attached.
            if let Some(layer) = self.app.0.layer() {
                unsafe { layer.setSublayers(None) };
            }
        }
        self.app.0.setHidden(!playing);
        self.resize()
    }

    pub(super) fn present(&self, frame: Frame) -> Result<()> {
        self.frame.replace(Some(frame));
        self.draw()
    }

    pub(super) fn resize(&self) -> Result<()> {
        let size = self.parent.bounds().size;
        let app_height = if self.playing.get() {
            size.height * APP_HEIGHT / (APP_HEIGHT + CONTROLS_HEIGHT)
        } else {
            0.0
        };
        let controls_height = size.height - app_height;
        let (app_y, controls_y) = if self.parent.isFlipped() {
            (0.0, app_height)
        } else {
            (controls_height, 0.0)
        };
        self.app
            .0
            .setFrame(NSRect::new(NSPoint::new(0.0, app_y), NSSize::new(size.width, app_height)));
        self.webview
            .setFrame(NSRect::new(NSPoint::new(0.0, controls_y), NSSize::new(size.width, controls_height)));
        self.draw()
    }

    fn draw(&self) -> Result<()> {
        let mut surface = self.surface.borrow_mut();
        let Some(surface) = surface.as_mut() else { return Ok(()) };
        let size = self.app.0.bounds().size;
        let scale = self.app.0.window().map_or(1.0, |window| window.backingScaleFactor());
        let (Some(width), Some(height)) = (
            NonZeroU32::new((size.width * scale).round() as u32),
            NonZeroU32::new((size.height * scale).round() as u32),
        ) else {
            return Ok(());
        };
        surface.resize(width, height).map_err(|error| anyhow!(error.to_string()))?;
        let mut buffer = surface.buffer_mut().map_err(|error| anyhow!(error.to_string()))?;
        if let Some(frame) = self.frame.borrow().as_ref() {
            frame.draw_into(&mut buffer, width.get(), height.get());
        } else {
            buffer.fill(0);
        }
        buffer.present().map_err(|error| anyhow!(error.to_string()))
    }

    pub(super) fn close(&self) {
        self.surface.take();
        self.frame.take();
        self.app.0.removeFromSuperview();
    }
}
