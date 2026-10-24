use std::{cell::Cell, ptr::NonNull, rc::Rc};

use anyhow::{Result, anyhow};
use block2::RcBlock;
use objc2::{
    MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained,
    runtime::{NSObjectProtocol, ProtocolObject},
};
use objc2_app_kit::{NSAutoresizingMaskOptions, NSView as HostView, NSWindowDidChangeOcclusionStateNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSPoint, NSRect, NSSize};
use raw_window_handle::{AppKitDisplayHandle, AppKitWindowHandle};
use tauri::webview::PlatformWebview;
use wgpu::{Instance, Surface, SurfaceTargetUnsafe};

use super::{APP_HEIGHT, CONTROLS_HEIGHT, request_redraw};

define_class!(
    #[unsafe(super = HostView)]
    #[thread_kind = MainThreadOnly]
    struct PresentationView;

    impl PresentationView {
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _rect: NSRect) {
            request_redraw();
        }

        #[unsafe(method(viewDidChangeBackingProperties))]
        fn backing_changed(&self) {
            unsafe { let _: () = msg_send![super(self), viewDidChangeBackingProperties]; }
            request_redraw();
        }
    }
);

pub struct View {
    parent: Retained<HostView>,
    webview: Retained<HostView>,
    app: Retained<PresentationView>,
    visibility_observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
    playing: Cell<bool>,
}

impl View {
    pub fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let main = MainThreadMarker::new().ok_or_else(|| anyhow!("Native view requires the main thread"))?;
        let webview = unsafe { Retained::<HostView>::retain(webview.inner().cast()) }.ok_or_else(|| anyhow!("WebView is unavailable"))?;
        let parent = unsafe { webview.superview() };
        let parent = parent.ok_or_else(|| anyhow!("WebView has no parent"))?;
        let app: Retained<PresentationView> =
            unsafe { msg_send![PresentationView::alloc(main), initWithFrame: NSRect::new(NSPoint::ZERO, NSSize::ZERO)] };
        app.setHidden(true);

        webview.setAutoresizingMask(NSAutoresizingMaskOptions::empty());
        parent.addSubview(&app);
        let window = parent.window().ok_or_else(|| anyhow!("WebView has no host window"))?;
        let callback = RcBlock::new(|_: NonNull<NSNotification>| request_redraw());
        // AppKit posts this on the main thread, including cached-layer uncovering.
        let visibility_observer = unsafe {
            NSNotificationCenter::defaultCenter().addObserverForName_object_queue_usingBlock(
                Some(NSWindowDidChangeOcclusionStateNotification),
                Some(&window),
                None,
                &callback,
            )
        };
        Ok(Rc::new(Self {
            parent,
            webview,
            app,
            visibility_observer,
            playing: Cell::new(false),
        }))
    }

    pub fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if !playing && let Some(layer) = self.app.layer() {
            unsafe { layer.setSublayers(None) };
        }
        self.app.setHidden(!playing);
        self.resize()
    }

    pub fn surface(&self, instance: &Instance) -> Result<Option<Surface<'static>>> {
        // wgpu creates a Metal sublayer; discard the previous one before reattaching.
        if let Some(layer) = self.app.layer() {
            unsafe { layer.setSublayers(None) };
        }
        let window = AppKitWindowHandle::new(NonNull::from(&*self.app).cast());
        // The UI owner drops this surface before removing the retained NSView.
        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(AppKitDisplayHandle::new().into()),
                raw_window_handle: window.into(),
            })?
        };
        Ok(Some(surface))
    }

    pub fn resize(&self) -> Result<()> {
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
            .setFrame(NSRect::new(NSPoint::new(0.0, app_y), NSSize::new(size.width, app_height)));
        self.webview
            .setFrame(NSRect::new(NSPoint::new(0.0, controls_y), NSSize::new(size.width, controls_height)));
        request_redraw();
        Ok(())
    }

    pub fn output_size(&self) -> Result<(u32, u32)> {
        if !self.playing.get() || self.app.isHiddenOrHasHiddenAncestor() || self.app.window().is_none_or(|window| window.isMiniaturized()) {
            return Ok((0, 0));
        }
        let size = self.app.convertRectToBacking(self.app.bounds()).size;
        Ok((size.width.round() as u32, size.height.round() as u32))
    }

    pub fn close(&self) {
        self.app.removeFromSuperview();
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe { NSNotificationCenter::defaultCenter().removeObserver(self.visibility_observer.as_ref()) };
    }
}
