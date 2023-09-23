use std::{
    cell::RefCell,
    fmt::Debug,
    num::NonZeroU32,
    rc::Rc,
    time::{Duration, Instant},
};

use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Window {
    window: winit::window::Window,
    event_loop: Option<EventLoop<()>>,
    surface: Surface,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new();

        let size = PhysicalSize::new(width, height);

        let window = WindowBuilder::new().with_inner_size(size).with_title("WIPI").build(&event_loop).unwrap();
        let context = unsafe { Context::new(&window) }.unwrap();
        let mut surface = unsafe { Surface::new(&context, &window) }.unwrap();

        surface
            .resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap())
            .unwrap();

        Self {
            window,
            event_loop: Some(event_loop),
            surface,
        }
    }

    pub fn paint(&mut self, data: &[u32]) {
        let mut buffer = self.surface.buffer_mut().unwrap();
        buffer.copy_from_slice(data);

        buffer.present().unwrap();
    }

    pub fn run<C, E>(self_: Rc<RefCell<Self>>, mut callback: C) -> !
    where
        C: FnMut(wie_base::Event) -> Result<(), E> + 'static,
        E: Debug,
    {
        let mut last_redraw = Instant::now();

        let event_loop = self_.borrow_mut().event_loop.take().unwrap();

        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let result = callback(wie_base::Event::Update);
                if let Err(x) = result {
                    tracing::error!(target: "wie", "{:?}", x);

                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::RedrawRequested(_) => {
                let result = callback(wie_base::Event::Redraw);
                if let Err(x) = result {
                    tracing::error!(target: "wie", "{:?}", x);

                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::RedrawEventsCleared => {
                let now = Instant::now();
                let next_frame_time = last_redraw + Duration::from_millis(16); // TODO hardcoded

                if now < next_frame_time {
                    *control_flow = ControlFlow::WaitUntil(next_frame_time)
                } else {
                    self_.borrow().window.request_redraw();
                    last_redraw = now;
                }
            }

            _ => *control_flow = ControlFlow::Wait,
        })
    }
}
