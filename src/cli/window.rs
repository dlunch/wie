use alloc::sync::Arc;
use std::sync::RwLock;

use anyhow::{Error, Result, anyhow, bail};
use futures::executor::block_on;
use wgpu::{FilterMode, Instance, InstanceDescriptor, PollType};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window as WinitWindow, WindowId},
};

use wie_backend::{Screen, canvas::Image};
use wie_renderer::{Frame, GpuContext, Presentation, Presenter, RenderTarget};
use wie_util::{Result as WieResult, WieError};

const MAX_DISPLAY_DIMENSION: u32 = 4096;
const MAX_DISPLAY_PIXELS: u32 = 4 * 1024 * 1024;

fn is_valid_display_size(width: u32, height: u32) -> bool {
    width > 0
        && height > 0
        && width <= MAX_DISPLAY_DIMENSION
        && height <= MAX_DISPLAY_DIMENSION
        && width.checked_mul(height).is_some_and(|pixels| pixels <= MAX_DISPLAY_PIXELS)
}

pub enum WindowInternalEvent {
    Resize(u32, u32),
    RequestRedraw,
    Paint(Frame),
    Error(String),
    Quit,
}

pub enum WindowCallbackEvent {
    Update,
    Redraw,
    Keydown(PhysicalKey),
    Keyup(PhysicalKey),
}

pub struct WindowHandle {
    display_size: Arc<RwLock<(u32, u32)>>,
    event_loop_proxy: EventLoopProxy<WindowInternalEvent>,
}

impl WindowHandle {
    pub fn send_quit_event(&self) {
        if let Err(error) = self.send_event(WindowInternalEvent::Quit) {
            tracing::warn!("Failed to send quit event: {error}");
        }
    }

    fn send_event(&self, event: WindowInternalEvent) -> WieResult<()> {
        self.event_loop_proxy
            .send_event(event)
            .map_err(|_| WieError::FatalError("Window event loop is closed".into()))
    }
}

impl Screen for WindowHandle {
    fn resize(&self, width: u32, height: u32) -> WieResult<()> {
        if !is_valid_display_size(width, height) {
            return Err(WieError::FatalError(format!("Invalid display size: {width}x{height}")));
        }
        *self
            .display_size
            .write()
            .map_err(|_| WieError::FatalError("Display size lock is poisoned".into()))? = (width, height);
        self.send_event(WindowInternalEvent::Resize(width, height))
    }

    fn request_redraw(&self) -> WieResult<()> {
        self.send_event(WindowInternalEvent::RequestRedraw)
    }

    fn paint(&self, image: &dyn Image) {
        if let Err(error) = self.send_event(WindowInternalEvent::Paint(Frame::from_image(image))) {
            tracing::warn!("Failed to send paint event: {error}");
        }
    }

    fn width(&self) -> u32 {
        self.display_size.read().unwrap().0
    }

    fn height(&self) -> u32 {
        self.display_size.read().unwrap().1
    }
}

pub struct WindowImpl {
    display_size: Arc<RwLock<(u32, u32)>>,
    event_loop: EventLoop<WindowInternalEvent>,
    filter: FilterMode,
}

impl WindowImpl {
    pub fn new(width: u32, height: u32, filter: FilterMode) -> Result<Self> {
        if !is_valid_display_size(width, height) {
            bail!("Invalid display size: {width}x{height}");
        }
        Ok(Self {
            display_size: Arc::new(RwLock::new((width, height))),
            event_loop: EventLoop::<WindowInternalEvent>::with_user_event().build()?,
            filter,
        })
    }

    pub fn handle(&self) -> WindowHandle {
        WindowHandle {
            display_size: self.display_size.clone(),
            event_loop_proxy: self.event_loop.create_proxy(),
        }
    }

    pub fn run<C>(self, callback: C) -> Result<()>
    where
        C: FnMut(WindowCallbackEvent) -> WieResult<()> + 'static,
    {
        self.event_loop.set_control_flow(ControlFlow::Poll);
        let (width, height) = *self.display_size.read().unwrap();
        let mut handler = ApplicationHandlerImpl {
            content_size: LogicalSize::new(width, height),
            native_scale_factor: 1.0,
            user_scale_factor: 1.0,
            scaled_size: PhysicalSize::new(width, height),
            window: None,
            instance: Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(
                self.event_loop.owned_display_handle(),
            ))),
            context: None,
            presenter: None,
            target: None,
            pending_frame: None,
            filter: self.filter,
            retry_used: false,
            waiting_for_output: false,
            proxy: self.event_loop.create_proxy(),
            callback,
            error: None,
        };
        self.event_loop.run_app(&mut handler)?;
        match handler.error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

struct ApplicationHandlerImpl<C> {
    content_size: LogicalSize<u32>,
    native_scale_factor: f64,
    user_scale_factor: f64,
    scaled_size: PhysicalSize<u32>,
    presenter: Option<Presenter<'static>>,
    target: Option<RenderTarget>,
    context: Option<GpuContext>,
    instance: Instance,
    window: Option<Arc<WinitWindow>>,
    pending_frame: Option<Frame>,
    filter: FilterMode,
    retry_used: bool,
    waiting_for_output: bool,
    proxy: EventLoopProxy<WindowInternalEvent>,
    callback: C,
    error: Option<Error>,
}

impl<C> ApplicationHandlerImpl<C>
where
    C: FnMut(WindowCallbackEvent) -> WieResult<()> + 'static,
{
    fn fail(&mut self, error: Error, event_loop: &ActiveEventLoop) {
        tracing::error!(target: "wie", "{error:#}");
        self.error = Some(error);
        event_loop.exit();
    }

    fn callback(&mut self, event: WindowCallbackEvent, event_loop: &ActiveEventLoop) {
        if let Err(error) = (self.callback)(event) {
            self.fail(error.into(), event_loop);
        }
    }

    fn create_presenter(&mut self) -> Result<()> {
        let window = self.window.as_ref().unwrap();
        let surface = self.instance.create_surface(window.clone())?;
        if self.context.is_none() {
            let proxy = self.proxy.clone();
            self.context = Some(block_on(GpuContext::new(&self.instance, &surface, move |error| {
                let _ = proxy.send_event(WindowInternalEvent::Error(format!("{error:#}")));
            }))?);
        }
        let context = self.context.as_ref().unwrap();
        let mut presenter = Presenter::new(context, surface, self.filter)?;
        let size = window.inner_size();
        presenter.resize(context, size.width, size.height)?;
        self.presenter = Some(presenter);
        Ok(())
    }

    fn redraw(&mut self) -> Result<()> {
        if self.waiting_for_output {
            return Ok(());
        }
        let Some(context) = &self.context else { return Ok(()) };
        let Some(presenter) = &mut self.presenter else { return Ok(()) };
        if let Some(frame) = self.pending_frame.take() {
            if self
                .target
                .as_ref()
                .is_none_or(|target| (target.width, target.height) != (frame.width, frame.height))
            {
                self.target = Some(RenderTarget::new(context, frame.width, frame.height)?);
            }
            self.target.as_ref().unwrap().upload(context, &frame);
        }
        match presenter.present(context, self.target.as_ref())? {
            Presentation::Presented => self.retry_used = false,
            Presentation::WaitForOutput => self.waiting_for_output = true,
            outcome @ (Presentation::Retry | Presentation::RecreateSurface) => {
                if self.retry_used {
                    bail!("GPU presentation failed after retry");
                }
                self.retry_used = true;
                if matches!(outcome, Presentation::RecreateSurface) {
                    self.presenter = None;
                    self.create_presenter()?;
                }
                self.window.as_ref().unwrap().request_redraw();
            }
        }
        Ok(())
    }

    fn resize_output(&mut self, size: PhysicalSize<u32>) -> Result<()> {
        self.waiting_for_output = size.width == 0 || size.height == 0;
        self.retry_used = false;
        if !self.waiting_for_output && size != self.scaled_size {
            let scale =
                (f64::from(size.width) / f64::from(self.content_size.width)).min(f64::from(size.height) / f64::from(self.content_size.height));
            self.update_scale_factor(None, Some(scale - self.native_scale_factor));
        }
        if let (Some(context), Some(presenter)) = (&self.context, &mut self.presenter) {
            presenter.resize(context, size.width, size.height)?;
        }
        self.window.as_ref().unwrap().request_redraw();
        Ok(())
    }

    fn update_scale_factor(&mut self, native: Option<f64>, user: Option<f64>) {
        if let Some(scale) = native {
            self.native_scale_factor = scale;
        }
        if let Some(scale) = user {
            self.user_scale_factor = scale;
        }
        if self.native_scale_factor + self.user_scale_factor < 0.1 {
            self.user_scale_factor = 0.0;
        }
        let scale = self.native_scale_factor + self.user_scale_factor;
        self.scaled_size = PhysicalSize::new(
            (f64::from(self.content_size.width) * scale).floor() as u32,
            (f64::from(self.content_size.height) * scale).floor() as u32,
        );
    }
}

impl<C> ApplicationHandler<WindowInternalEvent> for ApplicationHandlerImpl<C>
where
    C: FnMut(WindowCallbackEvent) -> WieResult<()> + 'static,
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        if let Some(context) = &self.context
            && let Err(error) = context.device.poll(PollType::Poll)
        {
            self.fail(error.into(), event_loop);
            return;
        }
        self.callback(WindowCallbackEvent::Update, event_loop);
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let result = (|| -> Result<()> {
            if self.window.is_none() {
                let attributes = WinitWindow::default_attributes().with_inner_size(self.content_size).with_title("WIE");
                let window = Arc::new(event_loop.create_window(attributes)?);
                self.update_scale_factor(Some(window.scale_factor()), None);
                let _ = window.request_inner_size(self.scaled_size);
                self.window = Some(window);
            }
            if self.presenter.is_none() {
                self.create_presenter()?;
            }
            self.waiting_for_output = false;
            self.retry_used = false;
            self.window.as_ref().unwrap().request_redraw();
            Ok(())
        })();
        if let Err(error) = result {
            self.fail(error, event_loop);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.presenter = None;
        self.waiting_for_output = true;
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: WindowInternalEvent) {
        match event {
            WindowInternalEvent::Resize(width, height) => {
                let size = LogicalSize::new(width, height);
                if self.content_size != size {
                    self.content_size = size;
                    self.update_scale_factor(None, None);
                    self.target = None;
                    self.pending_frame = None;
                    if let Some(window) = &self.window {
                        let _ = window.request_inner_size(self.scaled_size);
                        window.request_redraw();
                    }
                }
            }
            WindowInternalEvent::RequestRedraw => self.callback(WindowCallbackEvent::Redraw, event_loop),
            WindowInternalEvent::Paint(frame) => {
                self.pending_frame = Some(frame);
                if !self.waiting_for_output
                    && let Some(window) = &self.window
                {
                    window.request_redraw();
                }
            }
            WindowInternalEvent::Error(error) => self.fail(anyhow!(error), event_loop),
            WindowInternalEvent::Quit => event_loop.exit(),
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state,
                        repeat: false,
                        ..
                    },
                ..
            } => self.callback(
                match state {
                    ElementState::Pressed => WindowCallbackEvent::Keydown(physical_key),
                    ElementState::Released => WindowCallbackEvent::Keyup(physical_key),
                },
                event_loop,
            ),
            WindowEvent::RedrawRequested => {
                if let Err(error) = self.redraw() {
                    self.fail(error, event_loop);
                }
            }
            WindowEvent::Resized(size) => {
                if let Err(error) = self.resize_output(size) {
                    self.fail(error, event_loop);
                }
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                self.update_scale_factor(Some(scale_factor), None);
                let _ = inner_size_writer.request_inner_size(self.scaled_size);
            }
            WindowEvent::Occluded(occluded) => {
                self.waiting_for_output = occluded;
                if !occluded {
                    self.retry_used = false;
                    self.window.as_ref().unwrap().request_redraw();
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_valid_display_size;

    #[test]
    fn validates_display_size_bounds() {
        assert!(is_valid_display_size(176, 220));
        assert!(is_valid_display_size(2048, 2048));
        assert!(!is_valid_display_size(0, 220));
        assert!(!is_valid_display_size(176, 0));
        assert!(!is_valid_display_size(4097, 1));
        assert!(!is_valid_display_size(4096, 1025));
        assert!(!is_valid_display_size(u32::MAX, 2));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires an isolated X11 display and a graphics adapter"]
    fn retains_frame_when_only_the_output_window_is_resized() -> anyhow::Result<()> {
        use std::{
            sync::Arc,
            thread,
            time::{Duration, Instant},
        };

        use anyhow::{Context, ensure};
        use winit::{event_loop::EventLoop, platform::x11::EventLoopBuilderExtX11};
        use x11rb::{
            connection::Connection,
            protocol::xproto::{AtomEnum, ConfigureWindowAux, ConnectionExt, ImageFormat, MapState},
        };

        use wie_backend::{
            Screen,
            canvas::{ArgbPixel, VecImageBuffer},
        };

        use super::{WindowCallbackEvent, WindowImpl, WindowInternalEvent};

        let event_loop = EventLoop::with_user_event().with_x11().with_any_thread(true).build()?;
        let window = WindowImpl {
            display_size: Arc::new(std::sync::RwLock::new((240, 320))),
            event_loop,
            filter: wgpu::FilterMode::Nearest,
        };
        let handle = window.handle();
        let proxy = handle.event_loop_proxy.clone();
        let capture = thread::spawn(move || -> anyhow::Result<()> {
            let result = (|| -> anyhow::Result<()> {
                let (connection, screen) = x11rb::connect(None)?;
                let root = connection.setup().roots[screen].root;
                let started = Instant::now();
                let id = loop {
                    let mut found = None;
                    for id in connection.query_tree(root)?.reply()?.children {
                        let name = connection.get_property(false, id, AtomEnum::WM_NAME, AtomEnum::ANY, 0, 16)?.reply()?;
                        if name.value == b"WIE" && connection.get_window_attributes(id)?.reply()?.map_state == MapState::VIEWABLE {
                            found = Some(id);
                            break;
                        }
                    }
                    if let Some(id) = found {
                        break id;
                    }
                    ensure!(started.elapsed() < Duration::from_secs(10), "CLI window did not become visible");
                    thread::sleep(Duration::from_millis(10));
                };
                for (width, height) in [(480, 640), (640, 480)] {
                    connection
                        .configure_window(id, &ConfigureWindowAux::new().width(width).height(height))?
                        .check()?;
                    connection.flush()?;
                    let started = Instant::now();
                    loop {
                        let geometry = connection.get_geometry(id)?.reply()?;
                        if u32::from(geometry.width) == width && u32::from(geometry.height) == height {
                            let image = connection
                                .get_image(ImageFormat::Z_PIXMAP, id, 0, 0, geometry.width, geometry.height, u32::MAX)?
                                .reply()?;
                            ensure!(image.data.len() == width as usize * height as usize * 4, "Expected 32-bit X11 image");
                            let center = (height as usize / 2 * width as usize + width as usize / 2) * 4;
                            let border = (height as usize / 2 * width as usize + 1) * 4;
                            let filled = image.data[center..center + 3] == [0xff, 0xff, 0xff];
                            let letterboxed = width != 640 || image.data[border..border + 3] == [0, 0, 0];
                            if filled && letterboxed {
                                break;
                            }
                        }
                        ensure!(
                            started.elapsed() < Duration::from_secs(10),
                            "CLI did not present the retained frame at {width}x{height}"
                        );
                        thread::sleep(Duration::from_millis(10));
                    }
                }
                Ok(())
            })();
            let _ = proxy.send_event(WindowInternalEvent::Quit);
            result
        });
        let mut painted = false;
        let result = window.run(move |event| {
            if matches!(event, WindowCallbackEvent::Update) && !painted {
                painted = true;
                handle.paint(&VecImageBuffer::<ArgbPixel>::from_raw(240, 320, vec![0xffffffff; 240 * 320]));
            }
            Ok(())
        });
        capture
            .join()
            .map_err(|_| anyhow::anyhow!("CLI screenshot worker panicked"))?
            .context("CLI retained-frame capture failed")?;
        result
    }
}
