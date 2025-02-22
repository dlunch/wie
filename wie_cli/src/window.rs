use alloc::sync::Arc;
use core::{fmt::Debug, fmt::Formatter, num::NonZeroU32};
use std::fmt;

use fast_image_resize::ResizeAlg;
use fast_image_resize::{PixelType, ResizeOptions, SrcCropping};
use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window as WinitWindow, WindowId},
};

use wie_backend::{Screen, canvas::Image};

#[derive(Debug)]
pub enum WindowInternalEvent {
    RequestRedraw,
    Paint(Vec<u32>),
}

pub enum WindowCallbackEvent {
    Update,
    Redraw,
    Keydown(PhysicalKey),
    Keyup(PhysicalKey),
}

pub struct WindowHandle {
    width: u32,
    height: u32,
    event_loop_proxy: EventLoopProxy<WindowInternalEvent>,
}

impl WindowHandle {
    fn send_event(&self, event: WindowInternalEvent) -> wie_util::Result<()> {
        self.event_loop_proxy.send_event(event).unwrap();

        Ok(())
    }
}

impl Screen for WindowHandle {
    fn request_redraw(&self) -> wie_util::Result<()> {
        self.send_event(WindowInternalEvent::RequestRedraw)
    }

    fn paint(&mut self, image: &dyn Image) {
        let data = image
            .colors()
            .iter()
            .map(|x| ((x.a as u32) << 24) | ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32))
            .collect::<Vec<_>>();

        self.send_event(WindowInternalEvent::Paint(data)).unwrap()
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub struct WindowImpl {
    width: u32,
    height: u32,
    event_loop: EventLoop<WindowInternalEvent>,
}

impl WindowImpl {
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        let event_loop = EventLoop::<WindowInternalEvent>::with_user_event().build()?;

        Ok(Self { width, height, event_loop })
    }

    pub fn handle(&self) -> WindowHandle {
        WindowHandle {
            width: self.width,
            height: self.height,
            event_loop_proxy: self.event_loop.create_proxy(),
        }
    }

    pub fn run<C>(self, callback: C) -> anyhow::Result<()>
    where
        C: FnMut(WindowCallbackEvent) -> wie_util::Result<()> + 'static,
    {
        self.event_loop.set_control_flow(ControlFlow::Poll);

        const DEFAULT_USER_SCALE_FACTOR: f64 = 1.0;
        let orig_size = LogicalSize::new(self.width, self.height);
        let mut handler = ApplicationHandlerImpl {
            native_scale_factor: 1.0,
            user_scale_factor: DEFAULT_USER_SCALE_FACTOR,
            content_size: orig_size,
            scaled_size: orig_size.to_physical(1.0),
            window_size: Default::default(),
            scaler: Scaler::Native,
            scaled_image_buf: Default::default(),
            window: None,
            context: None,
            surface: None,
            callback: Box::new(callback),
            last_frame: None,
        };

        Ok(self.event_loop.run_app(&mut handler)?)
    }
}

enum Scaler {
    /// 1:1 native scaling.
    Native,
    /// hq2x, hq3x, hq4x scaling.
    Hqx { scale: i8 },
    /// Lanczos3 scaling
    Lanczos3 { scale: f64, resizer: fast_image_resize::Resizer },
}

impl fmt::Display for Scaler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Scaler::Native => f.write_str("Native")?,
            Scaler::Hqx { scale } => f.write_fmt(format_args!("Hq{}x", scale))?,
            Scaler::Lanczos3 { scale, resizer: _ } => f.write_fmt(format_args!("Lanczos3({})", scale))?,
        }
        Ok(())
    }
}

impl Scaler {
    fn new(scale: f64) -> Scaler {
        match scale {
            _ if (scale - 1.0).abs() < 1e-3 => Scaler::Native,
            _ => Scaler::Lanczos3 {
                scale,
                resizer: fast_image_resize::Resizer::new(),
            },
        }
    }

    #[allow(dead_code)]
    fn new_hqx(scale: f64) -> Scaler {
        match scale {
            _ if scale < 1.5 => Scaler::Native,
            _ if scale < 2.5 => Scaler::Hqx { scale: 2 },
            _ if scale < 3.5 => Scaler::Hqx { scale: 3 },
            _ => Scaler::Hqx { scale: 4 },
        }
    }

    fn scale(&self) -> f64 {
        match self {
            Scaler::Native => 1.0,
            Scaler::Hqx { scale } => *scale as f64,
            Scaler::Lanczos3 { scale, resizer: _ } => *scale,
        }
    }

    fn to_physical(&self, logical_size: LogicalSize<u32>) -> PhysicalSize<u32> {
        match self {
            Scaler::Native => PhysicalSize::new(logical_size.width, logical_size.height),
            Scaler::Hqx { scale } => PhysicalSize::new(logical_size.width * *scale as u32, logical_size.height * *scale as u32),
            Scaler::Lanczos3 { scale, resizer: _ } => PhysicalSize::new(
                (logical_size.width as f64 * *scale).floor() as u32,
                (logical_size.height as f64 * *scale).floor() as u32,
            ),
        }
    }

    fn scale_image(&mut self, dst: &mut Vec<u32>, src: &Vec<u32>, dst_size: PhysicalSize<u32>, src_size: LogicalSize<u32>) {
        match self {
            Scaler::Native => dst.copy_from_slice(src),
            Scaler::Hqx { scale } if *scale == 2 => hqx::hq2x(src.as_slice(), dst.as_mut_slice(), src_size.width as usize, src_size.height as usize),
            Scaler::Hqx { scale } if *scale == 3 => hqx::hq3x(src.as_slice(), dst.as_mut_slice(), src_size.width as usize, src_size.height as usize),
            Scaler::Hqx { scale } if *scale == 4 => hqx::hq4x(src.as_slice(), dst.as_mut_slice(), src_size.width as usize, src_size.height as usize),
            Scaler::Hqx { scale } => panic!("invalid hqx scale factor {}", scale),
            Scaler::Lanczos3 { scale: _, resizer } => {
                let (_, srcarr, _) = unsafe { src.align_to::<u8>() };
                let srcimg = fast_image_resize::images::ImageRef::new(src_size.width, src_size.height, srcarr, PixelType::U8x4).unwrap();
                let (_, dstarr, _) = unsafe { dst.as_mut_slice().align_to_mut::<u8>() };
                let mut dstimg = fast_image_resize::images::Image::from_slice_u8(dst_size.width, dst_size.height, dstarr, PixelType::U8x4).unwrap();
                resizer
                    .resize(
                        &srcimg,
                        &mut dstimg,
                        Some(&ResizeOptions {
                            #[cfg(debug_assertions)]
                            algorithm: ResizeAlg::Nearest,
                            #[cfg(not(debug_assertions))]
                            algorithm: ResizeAlg::Convolution(fast_image_resize::FilterType::Lanczos3),
                            cropping: SrcCropping::None,
                            mul_div_alpha: false,
                        }),
                    )
                    .unwrap();
            }
        }
    }
}

pub struct ApplicationHandlerImpl<C>
where
    C: FnMut(WindowCallbackEvent) -> wie_util::Result<()> + 'static,
{
    /// Native scale factor of the emulator window.
    native_scale_factor: f64,
    /// User specified scale factor.
    user_scale_factor: f64,
    /// Scaler config.
    /// Actual scaling factor = native_scale_factor + user_scale_factor
    scaler: Scaler,
    /// Temporary buffer for scaler.
    scaled_image_buf: Vec<u32>,

    /// content screen size.
    content_size: LogicalSize<u32>,
    /// Scaled screen size.
    /// Equals to orig_size * scale_factor.
    scaled_size: PhysicalSize<u32>,
    /// Size of the OS window.
    window_size: PhysicalSize<u32>,
    /// Last content screen image data.
    last_frame: Option<Vec<u32>>,

    window: Option<Arc<WinitWindow>>,
    context: Option<Context<Arc<WinitWindow>>>,
    surface: Option<Surface<Arc<WinitWindow>, Arc<WinitWindow>>>,
    callback: Box<C>,
}

impl<C> ApplicationHandlerImpl<C>
where
    C: FnMut(WindowCallbackEvent) -> wie_util::Result<()> + 'static,
{
    fn callback(&mut self, event: WindowCallbackEvent, event_loop: &ActiveEventLoop) {
        let result = (self.callback)(event);
        if let Err(x) = result {
            tracing::error!(target: "wie", "{}", x);

            event_loop.exit();
        }
    }

    /// Sets the native/user scale factor.
    /// After calling this you'll need to call [`Self::on_resize`] to update the surface accordingly.
    fn update_scale_factor(&mut self, native: Option<f64>, user: Option<f64>) {
        if let Some(f) = native {
            self.native_scale_factor = f
        }
        if let Some(f) = user {
            self.user_scale_factor = f
        }
        if self.native_scale_factor + self.user_scale_factor < 0.1 {
            tracing::info!(
                "scale factor too small(native {} + user {}), resetting user_scale_factor",
                self.native_scale_factor,
                self.user_scale_factor
            );
            self.user_scale_factor = 0.0;
        }

        self.scaler = Scaler::new(self.native_scale_factor + self.user_scale_factor);
        self.scaled_size = self.scaler.to_physical(self.content_size);
        self.scaled_image_buf = vec![0u32; self.scaled_size.width as usize * self.scaled_size.height as usize];
    }

    /// Updates the scaled content image surface's size.
    fn on_resize(&mut self) {
        tracing::info!(
            "on_resize scale=(native {}, actual {}), content={:?}, scaled={:?}, window={:?}",
            self.native_scale_factor,
            self.scaler.scale(),
            self.content_size,
            self.scaled_size,
            self.window_size
        );
        let surface = match self.surface.as_mut() {
            None => {
                self.surface = Some(Surface::new(self.context.as_ref().unwrap(), self.window.as_ref().unwrap().clone()).unwrap());
                self.surface.as_mut().unwrap()
            }
            Some(surface) => {
                let desired_len = self.scaled_size.width * self.scaled_size.height;
                if surface.buffer_mut().unwrap().len() == desired_len as usize {
                    // nothing to do
                    return;
                };
                surface
            }
        };

        surface
            .resize(
                NonZeroU32::new(self.scaled_size.width).unwrap(),
                NonZeroU32::new(self.scaled_size.height).unwrap(),
            )
            .unwrap();
        self.paint_last_frame();
    }

    /// Displays the last content frame to the window.
    fn paint_last_frame(&mut self) -> Option<()> {
        let data = self.last_frame.as_ref()?;
        if data.len() != self.content_size.width as usize * self.content_size.height as usize {
            return None;
        }
        let data_to_blit = if self.scaled_image_buf.len() == data.len() {
            data
        } else {
            self.scaler
                .scale_image(&mut self.scaled_image_buf, data, self.scaled_size, self.content_size);
            &self.scaled_image_buf
        };

        let mut win_buf = self.surface.as_mut().unwrap().buffer_mut().unwrap();
        if win_buf.len() == data_to_blit.len() {
            win_buf.copy_from_slice(data_to_blit);
        } else {
            tracing::warn!(
                "buffer size mismatch, skipping paint: {}, {} (content {:?}, scaled {:?}, win {:?})",
                win_buf.len(),
                data_to_blit.len(),
                self.content_size,
                self.scaled_size,
                self.window_size
            );
            return None;
        }
        win_buf.present().unwrap();
        Some(())
    }
}

impl<C> ApplicationHandler<WindowInternalEvent> for ApplicationHandlerImpl<C>
where
    C: FnMut(WindowCallbackEvent) -> wie_util::Result<()> + 'static,
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        self.callback(WindowCallbackEvent::Update, event_loop)
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Initialize the window.
        let window_attributes = WinitWindow::default_attributes()
            .with_inner_size(self.content_size.to_physical::<u32>(1.0))
            .with_title("WIE");
        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());
        let context = Context::new(window.clone()).unwrap();
        self.window = Some(window.clone());
        self.context = Some(context);
        self.window_size = window.inner_size();

        // After the window is initialized we resize the window again with the correct scale factor.
        self.update_scale_factor(Some(window.scale_factor()), Some(1.0));
        if let Some(new_size) = window.request_inner_size(self.scaled_size) {
            self.window_size = new_size;
        }
        self.on_resize();
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: WindowInternalEvent) {
        match event {
            WindowInternalEvent::RequestRedraw => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowInternalEvent::Paint(data) => {
                self.last_frame = Some(data);
                self.paint_last_frame();
            }
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
            } => match state {
                ElementState::Pressed => {
                    self.callback(WindowCallbackEvent::Keydown(physical_key), event_loop);
                }
                ElementState::Released => {
                    self.callback(WindowCallbackEvent::Keyup(physical_key), event_loop);
                }
            },
            WindowEvent::RedrawRequested => {
                self.callback(WindowCallbackEvent::Redraw, event_loop);
            }
            WindowEvent::Resized(new_size) => {
                tracing::debug!("WindowResized {:?}", new_size);
                self.window_size = new_size;
                if self.window_size != self.scaled_size {
                    // Determine the new scale factor.
                    let wscale = self.window_size.width as f64 / self.content_size.width as f64;
                    let hscale = self.window_size.height as f64 / self.content_size.height as f64;
                    let new_scale = wscale.min(hscale);
                    let new_user_scale = new_scale - self.native_scale_factor;
                    self.update_scale_factor(None, Some(new_user_scale));
                }
                self.on_resize();
            }
            WindowEvent::ScaleFactorChanged {
                scale_factor,
                mut inner_size_writer,
            } => {
                tracing::info!("ScaleFactorChanged {}", scale_factor);
                self.update_scale_factor(Some(scale_factor), None);
                let _ = inner_size_writer.request_inner_size(self.scaled_size);
                // Will receive WindowEvent::Resized soon, so no need to call self.on_resize().
            }
            _ => {}
        }
    }
}
