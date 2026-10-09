mod frame;
pub use frame::Frame;

#[cfg(target_os = "android")]
mod android;
#[cfg(target_vendor = "apple")]
mod apple;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "android")]
use android as platform;
#[cfg(target_vendor = "apple")]
use apple as platform;
#[cfg(target_os = "linux")]
use linux as platform;
#[cfg(target_os = "windows")]
use windows as platform;

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc,
    },
};

use anyhow::Result;
use spin::Mutex;
#[cfg(desktop)]
use tauri::{LogicalSize, PhysicalSize};
use tauri::{WebviewWindow, WindowEvent};

use wie_backend::{Screen, canvas::Image};
use wie_util::{Result as WieResult, WieError};

#[cfg(desktop)]
const APP_WIDTH: f64 = 360.0;
#[cfg(desktop)]
const APP_HEIGHT: f64 = 480.0;
#[cfg(desktop)]
const CONTROLS_HEIGHT: f64 = 260.0;

thread_local! {
    static VIEW: RefCell<Option<Rc<UiView>>> = const { RefCell::new(None) };
}

#[derive(Default)]
struct Pending {
    playing: bool,
    frame: Option<Frame>,
    scheduled: bool,
    error: Option<String>,
}

#[cfg(desktop)]
struct LibraryWindow {
    size: PhysicalSize<u32>,
    resizable: bool,
    maximized: bool,
}

struct UiView {
    native: Rc<platform::View>,
    pending: Arc<Mutex<Pending>>,
    window: WebviewWindow,
    playing: Cell<bool>,
    #[cfg(desktop)]
    library: RefCell<Option<LibraryWindow>>,
}

impl UiView {
    fn set_playing(&self, playing: bool) -> Result<()> {
        if !playing {
            self.native.set_playing(false)?;
        }
        let changed = self.playing.replace(playing) != playing;
        #[cfg(desktop)]
        if changed {
            if playing {
                let library = LibraryWindow {
                    size: self.window.inner_size()?,
                    resizable: self.window.is_resizable()?,
                    maximized: self.window.is_maximized()?,
                };
                let maximized = library.maximized;
                self.library.replace(Some(library));
                if maximized {
                    self.window.unmaximize()?;
                }
                self.window.set_resizable(false)?;
                self.window.set_size(LogicalSize::new(APP_WIDTH, APP_HEIGHT + CONTROLS_HEIGHT))?;
            } else {
                let library = self.library.take();
                if let Some(library) = library {
                    self.window.set_resizable(library.resizable)?;
                    self.window.set_size(library.size)?;
                    if library.maximized {
                        self.window.maximize()?;
                    }
                }
            }
        }
        #[cfg(mobile)]
        let _ = changed;
        if playing {
            self.native.set_playing(true)?;
        }
        Ok(())
    }
}

/// Thread-safe output proxy. Native objects never leave the UI thread.
#[derive(Clone)]
pub struct NativeView {
    window: WebviewWindow,
    pending: Arc<Mutex<Pending>>,
}

impl NativeView {
    /// Initialize from Tauri setup, before starting an app worker.
    pub fn new(window: WebviewWindow) -> Result<Self> {
        let pending = Arc::new(Mutex::new(Pending::default()));
        let state = pending.clone();
        let window_for_view = window.clone();
        let errors = pending.clone();
        window.with_webview(move |webview| {
            let install = move |native| {
                VIEW.set(Some(Rc::new(UiView {
                    native,
                    pending: state,
                    window: window_for_view,
                    playing: Cell::new(false),
                    #[cfg(desktop)]
                    library: RefCell::new(None),
                })));
            };
            #[cfg(not(target_os = "android"))]
            {
                let result = platform::View::new(webview).map(install);
                if let Err(error) = result {
                    errors.lock().error = Some(format!("{error:#}"));
                }
            }
            #[cfg(target_os = "android")]
            webview.jni_handle().exec(move |env, activity, webview| {
                let result = platform::View::new(env, activity, webview).map(install);
                if let Err(error) = result {
                    let _ = env.exception_clear();
                    errors.lock().error = Some(format!("{error:#}"));
                }
            });
        })?;

        let view = Self { window, pending };
        let listener = view.clone();
        view.window.on_window_event(move |event| match event {
            WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged { .. } => {
                if let Err(error) = listener.dispatch(|| {
                    let view = VIEW.with_borrow(Clone::clone);
                    if let Some(view) = view
                        && let Err(error) = view.native.resize()
                    {
                        report_error(error);
                    }
                }) {
                    log::error!("Native screen resize failed: {error}");
                }
            }
            WindowEvent::Destroyed => {
                listener.pending.lock().playing = false;
                if let Err(error) = listener.dispatch(|| {
                    let view = VIEW.take();
                    if let Some(view) = view {
                        view.native.close();
                    }
                }) {
                    log::error!("Native screen cleanup failed: {error}");
                }
            }
            _ => {}
        });
        Ok(view)
    }

    /// Called by the session worker, never from the UI thread or under its runtime lock.
    /// Success acknowledges both native layout changes and output resource cleanup.
    pub fn set_playing(&self, playing: bool) -> Result<()> {
        {
            let mut pending = self.pending.lock();
            pending.playing = false;
            pending.frame = None;
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let pending = self.pending.clone();
        self.dispatch(move || {
            let view = VIEW.with_borrow(Clone::clone);
            let result = match view {
                Some(view) => view.set_playing(playing),
                None => Err(anyhow::anyhow!(
                    pending.lock().error.take().unwrap_or_else(|| "Native screen is unavailable".into())
                )),
            };
            if result.is_ok() {
                pending.lock().playing = playing;
            }
            let _ = sender.send(result);
        })?;
        receiver.recv()?
    }

    pub fn present(&self, frame: Frame) -> Result<()> {
        {
            let mut pending = self.pending.lock();
            if !pending.playing {
                return Ok(());
            }
            pending.frame = Some(frame);
        }
        self.schedule()
    }

    /// The session loop consumes asynchronous native rendering failures here.
    pub fn take_error(&self) -> Option<String> {
        self.pending.lock().error.take()
    }

    fn schedule(&self) -> Result<()> {
        {
            let mut pending = self.pending.lock();
            if pending.scheduled {
                return Ok(());
            }
            pending.scheduled = true;
        }
        let pending = self.pending.clone();
        let result = self.dispatch(move || {
            let frame = {
                let mut pending = pending.lock();
                pending.scheduled = false;
                pending.frame.take()
            };
            let view = VIEW.with_borrow(Clone::clone);
            if let Some(view) = view
                && let Some(frame) = frame
                && let Err(error) = view.native.present(frame)
            {
                report_error(error);
            }
        });
        if result.is_err() {
            self.pending.lock().scheduled = false;
        }
        result
    }

    fn dispatch(&self, task: impl FnOnce() + Send + 'static) -> Result<()> {
        #[cfg(not(target_os = "android"))]
        self.window.run_on_main_thread(task)?;
        #[cfg(target_os = "android")]
        self.window.with_webview(move |webview| {
            webview.jni_handle().exec(move |_, _, _| task());
        })?;
        Ok(())
    }
}

fn report_error(error: anyhow::Error) {
    log::error!("Native screen failed: {error:#}");
    VIEW.with_borrow(|view| {
        if let Some(view) = view {
            view.pending.lock().error.get_or_insert_with(|| format!("{error:#}"));
        }
    });
}

pub struct NativeScreen {
    view: NativeView,
    width: AtomicU32,
    height: AtomicU32,
    should_redraw: Arc<AtomicBool>,
}

impl NativeScreen {
    pub fn new(view: NativeView, width: u32, height: u32, should_redraw: Arc<AtomicBool>) -> Self {
        Self {
            view,
            width: AtomicU32::new(width),
            height: AtomicU32::new(height),
            should_redraw,
        }
    }
}

impl Screen for NativeScreen {
    fn resize(&self, width: u32, height: u32) -> WieResult<()> {
        if width == 0 || height == 0 {
            return Err(WieError::FatalError(format!("Invalid display size: {width}x{height}")));
        }
        self.width.store(width, Ordering::Relaxed);
        self.height.store(height, Ordering::Relaxed);
        self.request_redraw()
    }

    fn request_redraw(&self) -> WieResult<()> {
        self.should_redraw.store(true, Ordering::Release);
        Ok(())
    }

    fn paint(&self, image: &dyn Image) {
        let frame = Frame {
            width: image.width(),
            height: image.height(),
            pixels: image
                .colors()
                .into_iter()
                .map(|color| (u32::from(color.r) << 16) | (u32::from(color.g) << 8) | u32::from(color.b))
                .collect(),
        };
        if let Err(error) = self.view.present(frame) {
            self.view.pending.lock().error.get_or_insert_with(|| error.to_string());
        }
    }

    fn width(&self) -> u32 {
        self.width.load(Ordering::Relaxed)
    }

    fn height(&self) -> u32 {
        self.height.load(Ordering::Relaxed)
    }
}
