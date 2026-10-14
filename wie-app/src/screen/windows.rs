use std::{
    cell::{Cell, RefCell},
    num::{NonZeroIsize, NonZeroU32},
    rc::Rc,
};

use anyhow::{Result, anyhow};
use raw_window_handle::{DisplayHandle, HandleError, HasWindowHandle, Win32WindowHandle, WindowHandle};
use softbuffer::{Context, Surface};
use tauri::webview::PlatformWebview;
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT},
        UI::{
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, GetClientRect, GetParent, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER, SetWindowPos, ShowWindow,
                WINDOW_EX_STYLE, WM_DPICHANGED, WM_ERASEBKGND, WM_NCDESTROY, WM_PAINT, WM_SIZE, WS_CHILD, WS_CLIPSIBLINGS,
            },
        },
    },
    core::w,
};

use super::{APP_HEIGHT, CONTROLS_HEIGHT, Frame, report_error};

struct AppWindow(Cell<Option<HWND>>);

impl HasWindowHandle for AppWindow {
    fn window_handle(&self) -> std::result::Result<WindowHandle<'_>, HandleError> {
        let hwnd = self.0.get().ok_or(HandleError::Unavailable)?;
        let raw = Win32WindowHandle::new(NonZeroIsize::new(hwnd.0 as isize).ok_or(HandleError::Unavailable)?);
        // The child HWND is retained by this UI-thread owner until its surface is released.
        Ok(unsafe { WindowHandle::borrow_raw(raw.into()) })
    }
}

pub(super) struct View {
    webview: PlatformWebview,
    parent: HWND,
    app: Rc<AppWindow>,
    surface: RefCell<Option<Surface<DisplayHandle<'static>, Rc<AppWindow>>>>,
    frame: RefCell<Option<Frame>>,
    playing: Cell<bool>,
    closed: Cell<bool>,
}

impl View {
    pub(super) fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
        let mut container = HWND::default();
        unsafe { webview.controller().ParentWindow(&mut container)? };
        let parent = unsafe { GetParent(container)? };
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!(""),
                WS_CHILD | WS_CLIPSIBLINGS,
                0,
                0,
                1,
                1,
                Some(parent),
                None,
                None,
                None,
            )?
        };
        let app = Rc::new(AppWindow(Cell::new(Some(hwnd))));
        let view = Rc::new(Self {
            webview,
            parent,
            app,
            surface: RefCell::new(None),
            frame: RefCell::new(None),
            playing: Cell::new(false),
            closed: Cell::new(false),
        });
        let pointer = Rc::as_ptr(&view) as usize;
        unsafe {
            SetWindowSubclass(hwnd, Some(app_proc), pointer, pointer).ok()?;
            SetWindowSubclass(parent, Some(parent_proc), pointer, pointer).ok()?;
        }
        Ok(view)
    }

    pub(super) fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        if playing && self.surface.borrow().is_none() {
            let context = Context::new(DisplayHandle::windows()).map_err(|error| anyhow!(error.to_string()))?;
            let surface = Surface::new(&context, self.app.clone()).map_err(|error| anyhow!(error.to_string()))?;
            self.surface.replace(Some(surface));
        } else if !playing {
            self.frame.take();
            self.surface.take();
        }
        self.resize()?;
        if let Some(hwnd) = self.app.0.get() {
            unsafe {
                let _ = ShowWindow(hwnd, if playing { SW_SHOW } else { SW_HIDE });
            }
        }
        Ok(())
    }

    pub(super) fn present(&self, frame: Frame) -> Result<()> {
        self.frame.replace(Some(frame));
        if let Some(hwnd) = self.app.0.get() {
            unsafe { InvalidateRect(Some(hwnd), None, false).ok()? };
        }
        Ok(())
    }

    pub(super) fn resize(&self) -> Result<()> {
        if self.closed.get() {
            return Ok(());
        }
        let mut rect = RECT::default();
        unsafe { GetClientRect(self.parent, &mut rect)? };
        let width = rect.right;
        let height = rect.bottom;
        let app_height = if self.playing.get() {
            (f64::from(height) * APP_HEIGHT / (APP_HEIGHT + CONTROLS_HEIGHT)).round() as i32
        } else {
            0
        };
        let controller = self.webview.controller();
        let mut container = HWND::default();
        unsafe {
            if let Some(hwnd) = self.app.0.get() {
                SetWindowPos(hwnd, None, 0, 0, width, app_height, SWP_NOACTIVATE | SWP_NOZORDER)?;
                InvalidateRect(Some(hwnd), None, false).ok()?;
            }
            controller.ParentWindow(&mut container)?;
            SetWindowPos(container, None, 0, app_height, width, height - app_height, SWP_NOACTIVATE | SWP_NOZORDER)?;
            controller.SetBounds(RECT {
                left: 0,
                top: 0,
                right: width,
                bottom: height - app_height,
            })?;
            controller.NotifyParentWindowPositionChanged()?;
        }
        Ok(())
    }

    fn draw(&self) -> Result<()> {
        let Some(hwnd) = self.app.0.get() else { return Ok(()) };
        let mut rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect)? };
        let (Some(width), Some(height)) = (NonZeroU32::new(rect.right as u32), NonZeroU32::new(rect.bottom as u32)) else {
            return Ok(());
        };
        let mut surface = self.surface.borrow_mut();
        let Some(surface) = surface.as_mut() else { return Ok(()) };
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
        if self.closed.replace(true) {
            return;
        }
        self.surface.take();
        let pointer = self as *const Self as usize;
        unsafe {
            let _ = RemoveWindowSubclass(self.parent, Some(parent_proc), pointer);
            if let Some(hwnd) = self.app.0.take() {
                let _ = RemoveWindowSubclass(hwnd, Some(app_proc), pointer);
                let _ = DestroyWindow(hwnd);
            }
        }
    }
}

impl Drop for View {
    fn drop(&mut self) {
        self.close();
    }
}

unsafe extern "system" fn parent_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM, _: usize, data: usize) -> LRESULT {
    // Native callbacks may destroy the window and remove the thread-local owner.
    let view = unsafe {
        Rc::increment_strong_count(data as *const View);
        Rc::from_raw(data as *const View)
    };
    // Wry restores full-window bounds during WM_SIZE; apply our split after its handler.
    let result = unsafe { DefSubclassProc(hwnd, message, wparam, lparam) };
    if matches!(message, WM_SIZE | WM_DPICHANGED) {
        if let Err(error) = view.resize() {
            report_error(error);
        }
    } else if message == WM_NCDESTROY {
        view.close();
    }
    result
}

unsafe extern "system" fn app_proc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM, _: usize, data: usize) -> LRESULT {
    let view = unsafe {
        Rc::increment_strong_count(data as *const View);
        Rc::from_raw(data as *const View)
    };
    match message {
        WM_PAINT => {
            let mut paint = PAINTSTRUCT::default();
            unsafe {
                let _ = BeginPaint(hwnd, &mut paint);
            }
            let result = view.draw();
            unsafe {
                let _ = EndPaint(hwnd, &paint);
            }
            if let Err(error) = result {
                report_error(error);
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_NCDESTROY => {
            view.surface.take();
            view.app.0.set(None);
            unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefSubclassProc(hwnd, message, wparam, lparam) },
    }
}
