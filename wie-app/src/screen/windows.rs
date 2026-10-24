use std::{cell::Cell, num::NonZeroIsize, rc::Rc};

use anyhow::{Result, anyhow};
use raw_window_handle::{Win32WindowHandle, WindowsDisplayHandle};
use tauri::webview::PlatformWebview;
use wgpu::{Instance, Surface, SurfaceTargetUnsafe};
use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT},
        UI::{
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::{
                CreateWindowExW, DestroyWindow, GetClientRect, GetParent, IsIconic, IsWindowVisible, SW_HIDE, SW_SHOW, SWP_NOACTIVATE, SWP_NOZORDER,
                SetWindowPos, ShowWindow, WINDOW_EX_STYLE, WM_DPICHANGED, WM_ERASEBKGND, WM_NCDESTROY, WM_PAINT, WM_SHOWWINDOW, WM_SIZE, WS_CHILD,
                WS_CLIPSIBLINGS,
            },
        },
    },
    core::w,
};

use super::{APP_HEIGHT, CONTROLS_HEIGHT, release_surface, report_error, request_redraw};

pub struct View {
    webview: PlatformWebview,
    parent: HWND,
    app: Cell<Option<HWND>>,
    playing: Cell<bool>,
    closed: Cell<bool>,
}

impl View {
    pub fn new(webview: PlatformWebview) -> Result<Rc<Self>> {
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
        let view = Rc::new(Self {
            webview,
            parent,
            app: Cell::new(Some(hwnd)),
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

    pub fn set_playing(&self, playing: bool) -> Result<()> {
        self.playing.set(playing);
        self.resize()?;
        if let Some(hwnd) = self.app.get() {
            unsafe {
                let _ = ShowWindow(hwnd, if playing { SW_SHOW } else { SW_HIDE });
            }
        }
        Ok(())
    }

    pub fn surface(&self, instance: &Instance) -> Result<Option<Surface<'static>>> {
        let Some(hwnd) = self.app.get() else { return Ok(None) };
        let window = Win32WindowHandle::new(NonZeroIsize::new(hwnd.0 as isize).ok_or_else(|| anyhow!("Native HWND is unavailable"))?);
        // The UI owner releases the surface before the child HWND is destroyed.
        let surface = unsafe {
            instance.create_surface_unsafe(SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(WindowsDisplayHandle::new().into()),
                raw_window_handle: window.into(),
            })?
        };
        Ok(Some(surface))
    }

    pub fn resize(&self) -> Result<()> {
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
            if let Some(hwnd) = self.app.get() {
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
        request_redraw();
        Ok(())
    }

    pub fn output_size(&self) -> Result<(u32, u32)> {
        let Some(hwnd) = self.app.get() else { return Ok((0, 0)) };
        if !self.playing.get() || unsafe { !IsWindowVisible(hwnd).as_bool() || IsIconic(self.parent).as_bool() } {
            return Ok((0, 0));
        }
        let mut rect = RECT::default();
        unsafe { GetClientRect(hwnd, &mut rect)? };
        Ok((rect.right as u32, rect.bottom as u32))
    }

    pub fn close(&self) {
        if self.closed.replace(true) {
            return;
        }
        let pointer = self as *const Self as usize;
        unsafe {
            let _ = RemoveWindowSubclass(self.parent, Some(parent_proc), pointer);
            if let Some(hwnd) = self.app.take() {
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
    if matches!(message, WM_SIZE | WM_DPICHANGED | WM_SHOWWINDOW) {
        if let Err(error) = view.resize() {
            report_error(error);
        }
    } else if message == WM_NCDESTROY {
        release_surface();
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
            request_redraw();
            unsafe {
                let _ = EndPaint(hwnd, &paint);
            }
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_NCDESTROY => {
            release_surface();
            view.app.set(None);
            unsafe { DefSubclassProc(hwnd, message, wparam, lparam) }
        }
        _ => unsafe { DefSubclassProc(hwnd, message, wparam, lparam) },
    }
}
