use alloc::sync::Arc;
use core::{fmt::Debug, num::NonZeroU32};

use softbuffer::{Context, Surface};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    keyboard::PhysicalKey,
    window::{Window as WinitWindow, WindowAttributes, WindowId},
};

use wie_backend::{canvas::Image, Screen};

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
    fn send_event(&self, event: WindowInternalEvent) -> anyhow::Result<()> {
        self.event_loop_proxy.send_event(event)?;

        Ok(())
    }
}

impl Screen for WindowHandle {
    fn request_redraw(&self) -> anyhow::Result<()> {
        self.send_event(WindowInternalEvent::RequestRedraw)
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn paint(&mut self, image: &dyn Image) {
        let data = image
            .colors()
            .iter()
            .map(|x| ((x.a as u32) << 24) | ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32))
            .collect::<Vec<_>>();

        self.send_event(WindowInternalEvent::Paint(data)).unwrap()
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

    pub fn run<C, E>(self, callback: C) -> anyhow::Result<()>
    where
        C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
        E: Debug,
    {
        self.event_loop.set_control_flow(ControlFlow::Poll);

        let size = PhysicalSize::new(self.width, self.height);
        let window_attributes = WinitWindow::default_attributes().with_inner_size(size).with_title("WIE");

        let mut handler = ApplicationHandlerImpl {
            window_attributes,
            window: None,
            surface: None,
            callback: Box::new(callback),
        };

        Ok(self.event_loop.run_app(&mut handler)?)
    }
}

pub struct ApplicationHandlerImpl<C, E>
where
    C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
    E: Debug,
{
    window_attributes: WindowAttributes,
    window: Option<Arc<WinitWindow>>,
    surface: Option<Surface<Arc<WinitWindow>, Arc<WinitWindow>>>,
    callback: Box<C>,
}

impl<C, E> ApplicationHandlerImpl<C, E>
where
    C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
    E: Debug,
{
    fn callback(&mut self, event: WindowCallbackEvent, event_loop: &ActiveEventLoop) {
        let result = (self.callback)(event);
        if let Err(x) = result {
            tracing::error!(target: "wie", "{:?}", x);

            event_loop.exit();
        }
    }
}

impl<C, E> ApplicationHandler<WindowInternalEvent> for ApplicationHandlerImpl<C, E>
where
    C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
    E: Debug,
{
    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: StartCause) {
        self.callback(WindowCallbackEvent::Update, event_loop)
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(event_loop.create_window(self.window_attributes.clone()).unwrap());

        let context = Context::new(window.clone()).unwrap();
        let mut surface = Surface::new(&context, window.clone()).unwrap();

        let size = window.inner_size();

        surface
            .resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap())
            .unwrap();

        self.window = Some(window);
        self.surface = Some(surface);
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, event: WindowInternalEvent) {
        match event {
            WindowInternalEvent::RequestRedraw => {
                self.window.as_ref().unwrap().request_redraw();
            }
            WindowInternalEvent::Paint(data) => {
                let mut buffer = self.surface.as_mut().unwrap().buffer_mut().unwrap();
                buffer.copy_from_slice(&data);

                buffer.present().unwrap();
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
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                self.callback(WindowCallbackEvent::Keydown(physical_key), event_loop);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key,
                        state: ElementState::Released,
                        ..
                    },
                ..
            } => {
                self.callback(WindowCallbackEvent::Keyup(physical_key), event_loop);
            }
            WindowEvent::RedrawRequested => {
                self.callback(WindowCallbackEvent::Redraw, event_loop);
            }
            _ => {}
        }
    }
}
