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

use anyhow::{Result, anyhow, bail};
use spin::Mutex;
#[cfg(desktop)]
use tauri::{LogicalSize, PhysicalSize};
use tauri::{WebviewWindow, WindowEvent};
use wgpu::{Device, FilterMode, Instance, InstanceDescriptor, PollType};

use wie_backend::{Screen, canvas::Image};
use wie_renderer::{Frame, GpuContext, Presentation, Presenter, RenderTarget};
use wie_util::{Result as WieResult, WieError};

use crate::settings::ScaleFilter;

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
    session: Option<u64>,
    filter: ScaleFilter,
    playing: bool,
    frame: Option<Frame>,
    redraw: bool,
    output_changed: bool,
    scheduled: bool,
    device: Option<Device>,
    error: Option<String>,
}

struct Output {
    presenter: Option<Presenter<'static>>,
    target: Option<RenderTarget>,
    context: Option<GpuContext>,
    instance: Instance,
    frame: Option<Frame>,
    filter: FilterMode,
    size: (u32, u32),
    retry_used: bool,
    waiting: bool,
    session: u64,
}

#[cfg(desktop)]
struct LibraryWindow {
    size: PhysicalSize<u32>,
    resizable: bool,
    maximized: bool,
}

struct UiView {
    output: RefCell<Option<Output>>,
    native: Rc<platform::View>,
    pending: Arc<Mutex<Pending>>,
    #[cfg(desktop)]
    window: WebviewWindow,
    playing: Cell<bool>,
    #[cfg(desktop)]
    library: RefCell<Option<LibraryWindow>>,
}

impl UiView {
    fn set_playing(&self, playing: bool) -> Result<()> {
        if !playing {
            self.output.take();
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
            let (session, filter) = {
                let pending = self.pending.lock();
                (pending.session.unwrap(), pending.filter)
            };
            #[cfg(target_os = "linux")]
            let descriptor = InstanceDescriptor::new_with_display_handle(Box::new(self.native.display_handle()?));
            #[cfg(not(target_os = "linux"))]
            let descriptor = InstanceDescriptor::new_without_display_handle();
            self.output.replace(Some(Output {
                presenter: None,
                target: None,
                context: None,
                instance: Instance::new(descriptor),
                frame: None,
                filter: match filter {
                    ScaleFilter::Linear => FilterMode::Linear,
                    ScaleFilter::Nearest => FilterMode::Nearest,
                },
                size: (0, 0),
                retry_used: false,
                waiting: false,
                session,
            }));
            self.native.set_playing(true)?;
            self.pending.lock().redraw = true;
        }
        Ok(())
    }

    fn draw(&self, frame: Option<Frame>, output_changed: bool) -> Result<()> {
        let mut output = self.output.borrow_mut();
        let Some(output) = output.as_mut() else { return Ok(()) };
        if frame.is_some() {
            output.frame = frame;
        }
        if output.waiting && output_changed {
            output.waiting = false;
            output.retry_used = false;
        }
        let size = self.native.output_size()?;
        // Retain only the newest CPU frame while uploads cannot be submitted.
        if size.0 == 0 || size.1 == 0 || output.waiting {
            output.waiting = true;
            return Ok(());
        }
        if output.presenter.is_none() {
            let Some(surface) = self.native.surface(&output.instance)? else {
                output.waiting = true;
                return Ok(());
            };
            if output.context.is_none() {
                let pending = self.pending.clone();
                let session = output.session;
                // Native wgpu requests complete without callbacks to the GTK/Tauri event loop.
                let context = futures::executor::block_on(GpuContext::new(&output.instance, &surface, move |error| {
                    report_gpu_error(&pending, session, error);
                }))?;
                {
                    let mut pending = self.pending.lock();
                    if pending.session == Some(session) {
                        pending.device = Some(context.device.clone());
                    }
                }
                output.context = Some(context);
            }
            output.presenter = Some(Presenter::new(output.context.as_ref().unwrap(), surface, output.filter)?);
            output.size = (0, 0);
        }
        let context = output.context.as_ref().unwrap();
        let presenter = output.presenter.as_mut().unwrap();
        if output.size != size {
            presenter.resize(context, size.0, size.1)?;
            output.size = size;
        }
        if let Some(frame) = output.frame.take() {
            if output
                .target
                .as_ref()
                .is_none_or(|target| (target.width, target.height) != (frame.width, frame.height))
            {
                output.target = Some(RenderTarget::new(context, frame.width, frame.height)?);
            }
            output.target.as_ref().unwrap().upload(context, &frame);
        }
        match presenter.present(context, output.target.as_ref())? {
            Presentation::Presented => output.retry_used = false,
            Presentation::WaitForOutput => output.waiting = true,
            result @ (Presentation::Retry | Presentation::RecreateSurface) => {
                if output.retry_used {
                    bail!("Native presentation failed after one retry");
                }
                output.retry_used = true;
                if matches!(result, Presentation::RecreateSurface) {
                    output.presenter.take();
                }
                self.pending.lock().redraw = true;
            }
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
        #[cfg(desktop)]
        let window_for_view = window.clone();
        let errors = pending.clone();
        window.with_webview(move |webview| {
            let install = move |native| {
                VIEW.set(Some(Rc::new(UiView {
                    native,
                    pending: state,
                    output: RefCell::new(None),
                    #[cfg(desktop)]
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
                {
                    let mut pending = listener.pending.lock();
                    pending.session = None;
                    pending.playing = false;
                    pending.device = None;
                    pending.frame = None;
                    pending.redraw = false;
                }
                if let Err(error) = listener.dispatch(|| {
                    let view = VIEW.take();
                    if let Some(view) = view {
                        view.output.take();
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
            if !playing {
                pending.session = None;
                pending.frame = None;
                pending.redraw = false;
                pending.output_changed = false;
                pending.scheduled = false;
                pending.device = None;
                pending.error = None;
            }
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let pending = self.pending.clone();
        self.dispatch(move || {
            let view = VIEW.with_borrow(Clone::clone);
            let result = match view {
                Some(view) => view.set_playing(playing),
                None => Err(anyhow!(
                    pending.lock().error.take().unwrap_or_else(|| "Native screen is unavailable".into())
                )),
            };
            if result.is_ok() {
                pending.lock().playing = playing;
            }
            let _ = sender.send(result);
        })?;
        receiver.recv()??;
        if playing {
            self.schedule()?;
        }
        Ok(())
    }

    pub fn present(&self, frame: Frame) -> Result<()> {
        self.pending.lock().frame = Some(frame);
        self.schedule()
    }

    pub fn begin_session(&self, session: u64, filter: ScaleFilter) {
        let mut pending = self.pending.lock();
        pending.session = Some(session);
        pending.filter = filter;
    }

    pub fn poll(&self) -> Result<()> {
        let (session, device) = {
            let pending = self.pending.lock();
            (pending.session, pending.device.clone())
        };
        if let Some(session) = session
            && let Some(device) = device
            && let Err(error) = device.poll(PollType::Poll)
        {
            report_gpu_error(&self.pending, session, error.into());
        }
        self.schedule()
    }

    /// The session loop consumes asynchronous native rendering failures here.
    pub fn take_error(&self) -> Option<String> {
        self.pending.lock().error.take()
    }

    fn schedule(&self) -> Result<()> {
        let session = {
            let mut pending = self.pending.lock();
            if !pending.playing || pending.scheduled || pending.error.is_some() || (pending.frame.is_none() && !pending.redraw) {
                return Ok(());
            }
            pending.scheduled = true;
            pending.session.unwrap()
        };
        let pending = self.pending.clone();
        let result = self.dispatch(move || {
            let (frame, output_changed) = {
                let mut pending = pending.lock();
                if pending.session != Some(session) {
                    return;
                }
                pending.scheduled = false;
                if !pending.playing {
                    return;
                }
                pending.redraw = false;
                (pending.frame.take(), std::mem::take(&mut pending.output_changed))
            };
            let view = VIEW.with_borrow(Clone::clone);
            if let Some(view) = view
                && let Err(error) = view.draw(frame, output_changed)
            {
                report_gpu_error(&pending, session, error);
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

fn report_gpu_error(pending: &Mutex<Pending>, session: u64, error: anyhow::Error) {
    let mut pending = pending.lock();
    if pending.session == Some(session) {
        log::error!("Native GPU failed: {error:#}");
        pending.error.get_or_insert_with(|| format!("{error:#}"));
    }
}

fn request_redraw() {
    VIEW.with_borrow(|view| {
        if let Some(view) = view {
            let mut pending = view.pending.lock();
            pending.redraw = true;
            pending.output_changed = true;
        }
    });
}

fn release_surface() {
    VIEW.with_borrow(|view| {
        if let Some(view) = view {
            if let Some(output) = view.output.borrow_mut().as_mut() {
                output.presenter.take();
                output.waiting = true;
            }
            view.pending.lock().redraw = false;
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
        let frame = Frame::from_image(image);
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
