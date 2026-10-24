#![cfg(target_os = "linux")]

use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use wgpu::{FilterMode, Instance, InstanceDescriptor, PollType};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoop},
    platform::x11::EventLoopBuilderExtX11,
    raw_window_handle::{HasWindowHandle, RawWindowHandle},
    window::{Window, WindowId},
};
use x11rb::protocol::xproto::{ConnectionExt, ImageFormat};

use wie_renderer::{Frame, GpuContext, Presentation, Presenter, RenderTarget};

struct SurfaceTest;

impl ApplicationHandler for SurfaceTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes().with_inner_size(PhysicalSize::new(64, 64)))
                .unwrap(),
        );
        let instance = Instance::new(InstanceDescriptor::new_with_display_handle(Box::new(event_loop.owned_display_handle())));
        let surface = instance.create_surface(window.clone()).unwrap();
        let errors = Arc::new(Mutex::new(Vec::new()));
        let sink = errors.clone();
        let context = block_on(GpuContext::new(&instance, &surface, move |error| {
            sink.lock().unwrap().push(error.to_string())
        }))
        .unwrap();
        eprintln!("Surface test adapter: {:?}", context.adapter.get_info());
        let mut presenter = Presenter::new(&context, surface, FilterMode::Nearest).unwrap();
        assert!(matches!(presenter.present(&context, None).unwrap(), Presentation::WaitForOutput));
        presenter.resize(&context, 64, 64).unwrap();
        assert!(matches!(presenter.present(&context, None).unwrap(), Presentation::Presented));
        let target = RenderTarget::new(&context, 2, 1).unwrap();
        target.upload(
            &context,
            &Frame {
                width: 2,
                height: 1,
                rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            },
        );
        assert!(matches!(presenter.present(&context, Some(&target)).unwrap(), Presentation::Presented));
        context.device.poll(PollType::wait_indefinitely()).unwrap();
        let RawWindowHandle::Xlib(handle) = window.window_handle().unwrap().as_raw() else {
            panic!("X11 surface test requires Xlib")
        };
        let (connection, _) = x11rb::connect(None).unwrap();
        let capture = |width, height| {
            let reply = connection
                .get_image(ImageFormat::Z_PIXMAP, handle.window as u32, 0, 0, width, height, u32::MAX)
                .unwrap()
                .reply()
                .unwrap();
            assert_eq!(reply.depth, 24);
            reply.data.chunks_exact(4).map(|pixel| [pixel[2], pixel[1], pixel[0]]).collect::<Vec<_>>()
        };
        let pixels = capture(64, 64);
        assert_eq!(pixels[0], [0, 0, 0]);
        assert_eq!(pixels[32 * 64 + 16], [255, 0, 0]);
        assert_eq!(pixels[32 * 64 + 48], [0, 0, 255]);
        presenter.resize(&context, 0, 0).unwrap();
        assert!(matches!(presenter.present(&context, Some(&target)).unwrap(), Presentation::WaitForOutput));
        let _ = window.request_inner_size(PhysicalSize::new(96, 64));
        presenter.resize(&context, 96, 64).unwrap();
        assert!(matches!(presenter.present(&context, Some(&target)).unwrap(), Presentation::Presented));
        context.device.poll(PollType::wait_indefinitely()).unwrap();
        let pixels = capture(96, 64);
        assert_eq!(pixels[0], [0, 0, 0]);
        assert_eq!(pixels[32 * 96 + 24], [255, 0, 0]);
        assert_eq!(pixels[32 * 96 + 72], [0, 0, 255]);
        let _ = window.request_inner_size(PhysicalSize::new(1, 1));
        presenter.resize(&context, 1, 1).unwrap();
        assert!(matches!(presenter.present(&context, Some(&target)).unwrap(), Presentation::Presented));
        context.device.poll(PollType::wait_indefinitely()).unwrap();
        assert!(matches!(capture(1, 1)[0], [255, 0, 0] | [0, 0, 255]));
        let errors = errors.lock().unwrap();
        assert!(errors.is_empty(), "{errors:?}");
        event_loop.exit();
    }

    fn window_event(&mut self, _: &ActiveEventLoop, _: WindowId, _: WindowEvent) {}
}

#[test]
#[ignore = "requires an isolated X11 display; run with xvfb-run"]
fn presents_and_resizes_a_retained_target_on_a_real_surface() {
    let event_loop = EventLoop::builder().with_x11().with_any_thread(true).build().unwrap();
    event_loop.run_app(&mut SurfaceTest).unwrap();
}
