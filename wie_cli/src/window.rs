use alloc::rc::Rc;
use core::{fmt::Debug, num::NonZeroU32};

use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
    keyboard::PhysicalKey,
    window::{Window as WinitWindow, WindowBuilder},
};

use wie_backend::{canvas::Canvas, Window};

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

pub struct WindowProxy {
    window: Rc<WinitWindow>,
    event_loop_proxy: EventLoopProxy<WindowInternalEvent>,
}

impl WindowProxy {
    fn send_event(&self, event: WindowInternalEvent) -> anyhow::Result<()> {
        self.event_loop_proxy.send_event(event)?;

        Ok(())
    }
}

impl Window for WindowProxy {
    fn request_redraw(&self) -> anyhow::Result<()> {
        self.send_event(WindowInternalEvent::RequestRedraw)
    }

    fn repaint(&self, canvas: &dyn Canvas) -> anyhow::Result<()> {
        let data = canvas
            .colors()
            .iter()
            .map(|x| ((x.a as u32) << 24) | ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32))
            .collect::<Vec<_>>();

        self.send_event(WindowInternalEvent::Paint(data))
    }

    fn width(&self) -> u32 {
        self.window.inner_size().width
    }

    fn height(&self) -> u32 {
        self.window.inner_size().height
    }
}

pub struct WindowImpl {
    window: Rc<WinitWindow>,
    event_loop: EventLoop<WindowInternalEvent>,
}

impl WindowImpl {
    pub fn new(width: u32, height: u32) -> anyhow::Result<Self> {
        let event_loop = EventLoopBuilder::<WindowInternalEvent>::with_user_event().build()?;

        let size = PhysicalSize::new(width, height);

        let builder = WindowBuilder::new().with_inner_size(size).with_title("WIE");

        let window = builder.build(&event_loop)?;

        Ok(Self {
            window: Rc::new(window),
            event_loop,
        })
    }

    pub fn proxy(&self) -> WindowProxy {
        WindowProxy {
            window: self.window.clone(),
            event_loop_proxy: self.event_loop.create_proxy(),
        }
    }

    fn callback<C, E>(event: WindowCallbackEvent, elwt: &EventLoopWindowTarget<WindowInternalEvent>, callback: &mut C)
    where
        C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
        E: Debug,
    {
        let result = callback(event);
        if let Err(x) = result {
            tracing::error!(target: "wie", "{:?}", x);

            elwt.exit();
        }
    }

    pub fn run<C, E>(self, mut callback: C) -> anyhow::Result<()>
    where
        C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
        E: Debug,
    {
        let context = Context::new(self.window.clone()).unwrap();
        let mut surface = Surface::new(&context, self.window.clone()).unwrap();

        let size = self.window.inner_size();

        surface
            .resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap())
            .unwrap();

        #[cfg(not(target_arch = "wasm32"))]
        let mut last_update = std::time::Instant::now();

        self.event_loop.run(move |event, elwt| match event {
            Event::UserEvent(x) => match x {
                WindowInternalEvent::RequestRedraw => {
                    self.window.request_redraw();
                }
                WindowInternalEvent::Paint(data) => {
                    let mut buffer = surface.buffer_mut().unwrap();
                    buffer.copy_from_slice(&data);

                    buffer.present().unwrap();
                }
            },

            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::KeyboardInput {
                    event:
                        KeyEvent {
                            physical_key,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    Self::callback(WindowCallbackEvent::Keydown(physical_key), elwt, &mut callback);
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
                    Self::callback(WindowCallbackEvent::Keyup(physical_key), elwt, &mut callback);
                }
                WindowEvent::RedrawRequested => {
                    Self::callback(WindowCallbackEvent::Redraw, elwt, &mut callback);
                }
                _ => {}
            },
            Event::AboutToWait => {
                #[cfg(target_arch = "wasm32")]
                {
                    Self::callback(WindowCallbackEvent::Update, elwt, &mut callback);
                    elwt.set_control_flow(ControlFlow::Wait);
                }
                #[cfg(not(target_arch = "wasm32"))]
                {
                    let now = std::time::Instant::now();
                    let next_update = last_update + std::time::Duration::from_millis(16);
                    if now < next_update {
                        elwt.set_control_flow(ControlFlow::WaitUntil(next_update));
                    } else {
                        Self::callback(WindowCallbackEvent::Update, elwt, &mut callback);

                        last_update = now;
                        let next_update = last_update + std::time::Duration::from_millis(16);
                        elwt.set_control_flow(ControlFlow::WaitUntil(next_update));
                    }
                }
            }
            _ => {}
        })?;

        Ok(())
    }
}
