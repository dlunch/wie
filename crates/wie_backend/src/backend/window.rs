use std::{
    num::NonZeroU32,
    time::{Duration, Instant},
};

use softbuffer::{Buffer, Context, Surface};
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Window {
    window: winit::window::Window,
    event_loop: EventLoop<()>,
}

impl Window {
    pub fn new() -> Self {
        let event_loop = EventLoop::new();
        let window = WindowBuilder::new().with_title("Hello world!").build(&event_loop).unwrap();

        Self { window, event_loop }
    }

    pub fn run<U, R>(self, mut update: U, mut render: R)
    where
        U: FnMut() + 'static,
        R: FnMut(&mut Buffer) + 'static,
    {
        let context = unsafe { Context::new(&self.window) }.unwrap();
        let mut surface = unsafe { Surface::new(&context, &self.window) }.unwrap();

        surface.resize(NonZeroU32::new(320).unwrap(), NonZeroU32::new(480).unwrap()).unwrap(); // TODO hardcoded

        let mut last_redraw = Instant::now();
        self.event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::MainEventsCleared => {
                update();
            }
            Event::RedrawRequested(_) => {
                let mut buffer = surface.buffer_mut().unwrap();
                render(&mut buffer);
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
        });
    }
}
