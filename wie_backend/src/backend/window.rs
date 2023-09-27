use core::{fmt::Debug, num::NonZeroU32};

use softbuffer::{Context, Surface};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::canvas::Canvas;

pub enum WindowCallbackEvent {
    Update,
    Event(wie_base::Event),
}

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

    pub fn paint(&mut self, canvas: &dyn Canvas) {
        let data = canvas
            .colors()
            .iter()
            .map(|x| ((x.a as u32) << 24) | ((x.r as u32) << 16) | ((x.g as u32) << 8) | (x.b as u32))
            .collect::<Vec<_>>();

        let mut buffer = self.surface.buffer_mut().unwrap();
        buffer.copy_from_slice(&data);

        buffer.present().unwrap();
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    fn callback<C, E>(event: WindowCallbackEvent, control_flow: &mut ControlFlow, callback: &mut C)
    where
        C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
        E: Debug,
    {
        let result = callback(event);
        if let Err(x) = result {
            tracing::error!(target: "wie", "{:?}", x);

            *control_flow = ControlFlow::Exit;
        }
    }

    pub fn event_loop(&mut self) -> EventLoop<()> {
        self.event_loop.take().unwrap()
    }

    pub fn run<C, E>(event_loop: EventLoop<()>, mut callback: C) -> !
    where
        C: FnMut(WindowCallbackEvent) -> Result<(), E> + 'static,
        E: Debug,
    {
        event_loop.run(move |event, _, control_flow| match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode,
                            state: ElementState::Pressed,
                            ..
                        },
                    ..
                } => {
                    Self::callback(
                        WindowCallbackEvent::Event(wie_base::Event::Keydown(scancode)),
                        control_flow,
                        &mut callback,
                    );
                }
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            scancode,
                            state: ElementState::Released,
                            ..
                        },
                    ..
                } => {
                    Self::callback(WindowCallbackEvent::Event(wie_base::Event::Keyup(scancode)), control_flow, &mut callback);
                }
                _ => {}
            },
            Event::MainEventsCleared => {
                Self::callback(WindowCallbackEvent::Update, control_flow, &mut callback);
            }
            Event::RedrawRequested(_) => {
                Self::callback(WindowCallbackEvent::Event(wie_base::Event::Redraw), control_flow, &mut callback);
            }

            _ => {}
        })
    }
}
