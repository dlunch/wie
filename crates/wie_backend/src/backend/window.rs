use std::{
    fmt::Debug,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use softbuffer::{Buffer, Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new(width: u32, height: u32) -> Self {
        let event_loop = EventLoop::new();
        let size = PhysicalSize::new(width, height);

        let window = WindowBuilder::new().with_inner_size(size).with_title("WIPI").build(&event_loop).unwrap();

        Self { window, event_loop }
    }

    pub fn run<U, R, E>(self, mut update: U, mut render: R) -> !
    where
        U: FnMut() -> Result<(), E> + 'static,
        R: FnMut(&mut Buffer) -> Result<(), E> + 'static,
        E: Debug,
    {
        let context = unsafe { Context::new(&self.window) }.unwrap();
        let mut surface = unsafe { Surface::new(&context, &self.window) }.unwrap();

        let size = self.window.inner_size();

        surface
            .resize(NonZeroU32::new(size.width).unwrap(), NonZeroU32::new(size.height).unwrap())
            .unwrap();

        let mut last_redraw = Instant::now();
        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                let result = update();
                if let Err(x) = result {
                    log::error!(target: "wie", "{:?}", x);

                    *control_flow = ControlFlow::Exit;
                }
            }
            Event::RedrawRequested(_) => {
                let mut buffer = surface.buffer_mut().unwrap();
                let result = render(&mut buffer);
                if let Err(x) = result {
                    log::error!(target: "wie", "{:?}", x);

                    *control_flow = ControlFlow::Exit;
                }
                buffer.present().unwrap();
            }
            Event::RedrawEventsCleared => {
                let now = Instant::now();
                let next_frame_time = last_redraw + Duration::from_millis(16); // TODO hardcoded

                if now < next_frame_time {
                    *control_flow = ControlFlow::WaitUntil(next_frame_time)
                } else {
                    self.window.request_redraw();
                    last_redraw = now;
                }
            }

            _ => *control_flow = ControlFlow::Wait,
        })
    }
}
