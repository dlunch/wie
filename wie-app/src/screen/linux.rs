use std::{
    cell::Cell,
    ffi::{c_ulong, c_void},
    num::NonZeroUsize,
    ptr::NonNull,
    rc::Rc,
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use gtk::{
    gdk,
    glib::{self, subclass::prelude::*, translate::*},
    prelude::*,
    subclass::prelude::*,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, RawDisplayHandle, RawWindowHandle, WaylandDisplayHandle, WaylandWindowHandle, XlibDisplayHandle,
    XlibWindowHandle,
};
use tauri::webview::PlatformWebview;
use wayland_client::{
    Connection, Proxy,
    backend::{Backend, ObjectId},
    protocol::wl_surface::WlSurface,
};
use wgpu::{Instance, Surface, SurfaceTargetUnsafe};

use super::{APP_HEIGHT, APP_WIDTH, CONTROLS_HEIGHT, release_surface, report_error, request_redraw};

#[link(name = "gdk-3")]
unsafe extern "C" {
    fn gdk_x11_display_get_xdisplay(display: *mut gdk::ffi::GdkDisplay) -> *mut c_void;
    fn gdk_x11_screen_get_screen_number(screen: *mut gdk::ffi::GdkScreen) -> i32;
    fn gdk_x11_window_get_xid(window: *mut gdk::ffi::GdkWindow) -> c_ulong;
    fn gdk_wayland_display_get_wl_display(display: *mut gdk::ffi::GdkDisplay) -> *mut c_void;
    fn gdk_wayland_window_get_wl_surface(window: *mut gdk::ffi::GdkWindow) -> *mut c_void;
}

// The UI-owned GTK view keeps this display live until its wgpu instance is released.
#[derive(Debug)]
pub struct NativeDisplay {
    address: NonZeroUsize,
    wayland: bool,
    screen: i32,
}

impl HasDisplayHandle for NativeDisplay {
    fn display_handle(&self) -> std::result::Result<DisplayHandle<'_>, HandleError> {
        let pointer = NonNull::new(self.address.get() as *mut c_void).unwrap();
        let raw = if self.wayland {
            RawDisplayHandle::Wayland(WaylandDisplayHandle::new(pointer))
        } else {
            RawDisplayHandle::Xlib(XlibDisplayHandle::new(Some(pointer), self.screen))
        };
        Ok(unsafe { DisplayHandle::borrow_raw(raw) })
    }
}

mod widget {
    use super::*;

    #[derive(Default)]
    pub struct NativeWidget;

    #[glib::object_subclass]
    impl ObjectSubclass for NativeWidget {
        const NAME: &'static str = "WieNativeScreen";
        type Type = super::NativeWidget;
        type ParentType = gtk::Widget;
    }

    impl ObjectImpl for NativeWidget {
        fn constructed(&self) {
            self.parent_constructed();
            let widget = self.obj();
            widget.set_has_window(true);
            widget.set_app_paintable(true);
            // GTK must not attach a Cairo backing buffer over the GPU-owned output.
            unsafe { gtk::ffi::gtk_widget_set_double_buffered(widget.as_ptr().cast(), 0) };
        }
    }

    impl WidgetImpl for NativeWidget {
        fn realize(&self) {
            let result = (|| -> Result<()> {
                let widget = self.obj();
                let parent = widget.parent_window().context("Native screen has no GTK parent window")?;
                let allocation = widget.allocation();
                let wayland = parent.display().type_().name().contains("Wayland");
                let native_parent = if wayland {
                    parent.screen().root_window().unwrap()
                } else {
                    parent.clone()
                };
                let window = gdk::Window::new(
                    Some(&native_parent),
                    &gdk::WindowAttr {
                        x: Some(allocation.x()),
                        y: Some(allocation.y()),
                        width: allocation.width(),
                        height: allocation.height(),
                        window_type: if wayland { gdk::WindowType::Subsurface } else { gdk::WindowType::Child },
                        event_mask: gdk::EventMask::EXPOSURE_MASK,
                        visual: widget.visual(),
                        ..Default::default()
                    },
                );
                ensure!(window.ensure_native(), "GTK could not create an independent native output window");
                if wayland {
                    window.set_transient_for(&parent);
                }
                widget.set_realized(true);
                widget.set_window(window.clone());
                widget.register_window(&window);
                Ok(())
            })();
            if let Err(error) = result {
                report_error(error);
            }
        }

        fn unrealize(&self) {
            release_surface();
            self.parent_unrealize();
        }

        fn map(&self) {
            self.parent_map();
            request_redraw();
        }

        fn unmap(&self) {
            // GDK replaces the Wayland wl_surface even when only hiding the widget.
            release_surface();
            self.parent_unmap();
        }

        fn size_allocate(&self, allocation: &gtk::Allocation) {
            let widget = self.obj();
            widget.set_allocation(allocation);
            if let Some(window) = widget.window() {
                window.move_resize(allocation.x(), allocation.y(), allocation.width(), allocation.height());
                request_redraw();
            }
        }

        fn draw(&self, _: &gtk::cairo::Context) -> glib::Propagation {
            request_redraw();
            glib::Propagation::Stop
        }
    }
}

glib::wrapper! {
    pub struct NativeWidget(ObjectSubclass<widget::NativeWidget>) @extends gtk::Widget;
}

pub struct View {
    area: NativeWidget,
    window: gtk::Window,
    library_default_size: Cell<Option<(i32, i32)>>,
    buffer_scale: Cell<i32>,
}

impl View {
    pub fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let webview = webview.inner();
        let parent = webview.parent().ok_or_else(|| anyhow!("WebView has no GTK parent"))?;
        let container = parent.downcast::<gtk::Box>().map_err(|_| anyhow!("WebView parent is not a GtkBox"))?;
        let window = container
            .toplevel()
            .and_downcast::<gtk::Window>()
            .ok_or_else(|| anyhow!("GTK window is unavailable"))?;
        let area: NativeWidget = glib::Object::new();
        area.set_size_request(-1, APP_HEIGHT as i32);
        area.set_hexpand(true);
        area.set_no_show_all(true);
        area.connect_scale_factor_notify(|_| request_redraw());
        container.pack_start(&area, false, false, 0);
        container.reorder_child(&area, 0);
        Ok(Rc::new(Self {
            area,
            window,
            library_default_size: Cell::new(None),
            buffer_scale: Cell::new(0),
        }))
    }

    pub fn display_handle(&self) -> Result<NativeDisplay> {
        let display = self.window.display();
        let name = display.type_().name();
        let (address, wayland, screen) = unsafe {
            if name.contains("Wayland") {
                (gdk_wayland_display_get_wl_display(display.to_glib_none().0), true, 0)
            } else if name.contains("X11") {
                (
                    gdk_x11_display_get_xdisplay(display.to_glib_none().0),
                    false,
                    gdk_x11_screen_get_screen_number(display.default_screen().to_glib_none().0),
                )
            } else {
                bail!("Unsupported GTK display backend: {name}");
            }
        };
        Ok(NativeDisplay {
            address: NonZeroUsize::new(address as usize).context("GTK native display is unavailable")?,
            wayland,
            screen,
        })
    }

    pub fn surface(&self, instance: &Instance) -> Result<Option<Surface<'static>>> {
        if !self.area.is_mapped() {
            return Ok(None);
        }
        let window = self.area.window().context("GTK native output window is unavailable")?;
        let display = self.display_handle()?;
        self.update_buffer_scale(&window, &display)?;
        let raw_display = display.display_handle()?.as_raw();
        let raw_window = unsafe {
            if display.wayland {
                let pointer =
                    NonNull::new(gdk_wayland_window_get_wl_surface(window.to_glib_none().0)).context("GTK Wayland surface is unavailable")?;
                RawWindowHandle::Wayland(WaylandWindowHandle::new(pointer))
            } else {
                let xid = gdk_x11_window_get_xid(window.to_glib_none().0);
                ensure!(xid != 0, "GTK X11 output window is unavailable");
                RawWindowHandle::Xlib(XlibWindowHandle::new(xid))
            }
        };
        // unmap/unrealize synchronously release the presenter before GDK invalidates these handles.
        Ok(Some(unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(raw_display),
                raw_window_handle: raw_window,
            })
        }?))
    }

    pub fn output_size(&self) -> Result<(u32, u32)> {
        if !self.area.is_mapped() {
            return Ok((0, 0));
        }
        let scale = self.area.scale_factor() as u32;
        if self.buffer_scale.get() != scale as i32 {
            self.update_buffer_scale(&self.area.window().unwrap(), &self.display_handle()?)?;
        }
        Ok((self.area.allocated_width() as u32 * scale, self.area.allocated_height() as u32 * scale))
    }

    fn update_buffer_scale(&self, window: &gdk::Window, display: &NativeDisplay) -> Result<()> {
        let scale = window.scale_factor();
        if display.wayland {
            // External buffers bypass GDK's Cairo/EGL buffer-scale initialization.
            unsafe {
                let pointer =
                    NonNull::new(gdk_wayland_window_get_wl_surface(window.to_glib_none().0)).context("GTK Wayland surface is unavailable")?;
                let connection = Connection::from_backend(Backend::from_foreign_display((display.address.get() as *mut c_void).cast()));
                let id = ObjectId::from_ptr(WlSurface::interface(), pointer.as_ptr().cast())?;
                WlSurface::from_id(&connection, id)?.set_buffer_scale(scale);
            }
        }
        self.buffer_scale.set(scale);
        Ok(())
    }

    pub fn set_playing(&self, playing: bool) -> Result<()> {
        if playing {
            if self.library_default_size.get().is_none() {
                self.library_default_size.set(Some(self.window.default_size()));
            }
            // GTK uses the default size as the minimum while the window is non-resizable.
            self.window.set_default_size(APP_WIDTH as i32, (APP_HEIGHT + CONTROLS_HEIGHT) as i32);
            self.area.show();
        } else {
            release_surface();
            self.area.hide();
            if let Some((width, height)) = self.library_default_size.take() {
                self.window.set_default_size(width, height);
            }
        }
        Ok(())
    }

    pub fn resize(&self) -> Result<()> {
        request_redraw();
        Ok(())
    }

    pub fn close(&self) {
        release_surface();
        if let Some(parent) = self.area.parent().and_downcast::<gtk::Container>() {
            parent.remove(&self.area);
        }
    }
}
