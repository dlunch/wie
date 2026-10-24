use std::{
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Result, anyhow, ensure};
use gtk::prelude::*;
use tauri::{LogicalSize, WebviewUrl, WebviewWindowBuilder};
use wgpu::{Color, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp};
use wie_renderer::Frame;

use crate::{
    screen::{NativeView, VIEW, request_redraw},
    settings::ScaleFilter,
};

#[test]
#[ignore = "requires an isolated X11 or Wayland display; creates a real Tauri window"]
fn native_presentation_retains_content_across_output_lifecycle() {
    let mut context = tauri::generate_context!();
    context.config_mut().app.windows.clear();
    let app = tauri::Builder::default().any_thread().build(context).unwrap();
    let window = WebviewWindowBuilder::new(&app, "main", WebviewUrl::External("about:blank".parse().unwrap()))
    .initialization_script("document.addEventListener('DOMContentLoaded', () => { document.body.style.cssText = 'background:#343536;color:white'; document.body.textContent = 'This project is based on wie, created by Inseok Lee.'; });")
    .inner_size(640.0, 600.0)
    .build()
    .unwrap();
    let view = NativeView::new(window.clone()).unwrap();
    let (completed, completion) = mpsc::channel();
    let handle = app.handle().clone();
    let worker = thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<()> {
            view.begin_session(1, ScaleFilter::Nearest);
            let start = Instant::now();
            view.set_playing(true)?;
            settle(&view)?;
            println!("NATIVE_STARTUP elapsed={:?}", start.elapsed());
            on_ui(&view, || {
                VIEW.with_borrow(|view| {
                    let view = view.as_ref().unwrap();
                    let output = view.output.borrow();
                    let output = output.as_ref().unwrap();
                    assert!(output.presenter.is_some());
                    assert!(output.target.is_none());
                    println!("NATIVE_ADAPTER {:?}", output.context.as_ref().unwrap().adapter.get_info());
                });
            })?;

            view.present(Frame {
                width: 2,
                height: 1,
                rgba: vec![255, 0, 0, 255, 0, 0, 255, 255],
            })?;
            settle(&view)?;
            capture(&view, "cpu", &[(255, 0, 0), (0, 0, 255)])?;

            on_ui(&view, || {
                VIEW.with_borrow(|view| {
                    let view = view.as_ref().unwrap();
                    let output = view.output.borrow();
                    let output = output.as_ref().unwrap();
                    let context = output.context.as_ref().unwrap();
                    let target = output.target.as_ref().unwrap();
                    let mut encoder = context.device.create_command_encoder(&Default::default());
                    {
                        let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &target.view,
                                depth_slice: None,
                                resolve_target: None,
                                ops: Operations {
                                    load: LoadOp::Clear(Color {
                                        r: 0.0,
                                        g: 1.0,
                                        b: 0.0,
                                        a: 0.25,
                                    }),
                                    store: StoreOp::Store,
                                },
                            })],
                            ..Default::default()
                        });
                    }
                    context.queue.submit([encoder.finish()]);
                });
                request_redraw();
            })?;
            settle(&view)?;
            capture(&view, "gpu", &[(0, 255, 0)])?;
            window.set_size(LogicalSize::new(720.0, 740.0))?;
            settle(&view)?;
            capture(&view, "gpu-resized", &[(0, 255, 0)])?;

            on_ui(&view, || VIEW.with_borrow(|view| view.as_ref().unwrap().native.area.hide()))?;
            for width in 3..=7 {
                view.present(Frame {
                    width,
                    height: 1,
                    rgba: [0, 0, 255, 255].repeat(width as usize),
                })?;
                settle(&view)?;
            }
            on_ui(&view, || {
                VIEW.with_borrow(|view| {
                    let view = view.as_ref().unwrap();
                    let output = view.output.borrow();
                    let output = output.as_ref().unwrap();
                    assert!(output.presenter.is_none());
                    assert_eq!(output.target.as_ref().unwrap().width, 2);
                    assert_eq!(output.frame.as_ref().unwrap().width, 7);
                    assert!(output.waiting);
                })
            })?;
            on_ui(&view, || VIEW.with_borrow(|view| view.as_ref().unwrap().native.area.show()))?;
            settle(&view)?;
            capture(&view, "latest-after-show", &[(0, 0, 255)])?;
            on_ui(&view, || {
                VIEW.with_borrow(|view| {
                    let area = &view.as_ref().unwrap().native.area;
                    area.hide();
                    area.unrealize();
                    area.show();
                })
            })?;
            settle(&view)?;
            capture(&view, "recreated", &[(0, 0, 255)])?;
            let old_device = view.pending.lock().device.clone().unwrap();
            let invalid_shader = |device: &wgpu::Device| {
                let _ = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("intentional native error delivery fixture"),
                    source: wgpu::ShaderSource::Wgsl("not valid WGSL".into()),
                });
            };
            view.set_playing(false)?;
            invalid_shader(&old_device);
            ensure!(view.take_error().is_none(), "stopped device error leaked");
            view.begin_session(2, ScaleFilter::Linear);
            view.set_playing(true)?;
            settle(&view)?;
            invalid_shader(&old_device);
            ensure!(view.take_error().is_none(), "old device error reached a new session");
            let device = view.pending.lock().device.clone().unwrap();
            invalid_shader(&device);
            view.poll()?;
            ensure!(view.take_error().is_some(), "idle GPU validation error was not delivered");
            view.set_playing(false)?;
            on_ui(&view, || {
                VIEW.with_borrow(|view| assert!(view.as_ref().unwrap().output.borrow().is_none()))
            })?;
            Ok(())
        }));
        completed.send(result).unwrap();
        handle.exit(0);
    });
    app.run_return(|_, _| {});
    worker.join().unwrap();
    match completion.recv().unwrap() {
        Ok(result) => result.unwrap(),
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

fn settle(view: &NativeView) -> Result<()> {
    for _ in 0..30 {
        view.poll()?;
        if let Some(error) = view.take_error() {
            return Err(anyhow!(error));
        }
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

fn on_ui(view: &NativeView, task: impl FnOnce() + Send + 'static) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    view.dispatch(move || {
        task();
        let _ = sender.send(());
    })?;
    receiver.recv_timeout(Duration::from_secs(10))?;
    Ok(())
}

fn capture(view: &NativeView, stage: &str, colors: &[(u8, u8, u8)]) -> Result<()> {
    let (sender, receiver) = mpsc::channel();
    view.dispatch(move || {
        let display = gtk::gdk::Display::default().unwrap();
        let root = display.default_screen().root_window().unwrap();
        let (viewport, source, scale) = VIEW.with_borrow(|view| {
            let view = view.as_ref().unwrap();
            let output = view.output.borrow();
            let target = output.as_ref().unwrap().target.as_ref().unwrap();
            let scale = view.native.area.scale_factor() as u32;
            let viewport = view.native.output_size().unwrap();
            assert_eq!(
                viewport,
                (
                    view.native.area.allocated_width() as u32 * scale,
                    view.native.area.allocated_height() as u32 * scale,
                )
            );
            (viewport, (target.width, target.height), scale)
        });
        sender
            .send((
                display.type_().name().contains("Wayland"),
                root.width(),
                root.height(),
                root.scale_factor(),
                viewport,
                source,
                scale,
            ))
            .unwrap();
    })?;
    let (wayland, width, height, root_scale, viewport, source, scale) = receiver.recv_timeout(Duration::from_secs(10))?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join(format!("{stage}.png"));
    let status = if wayland {
        std::process::Command::new("spectacle")
            .args(["-b", "-n", "-f", "-o"])
            .arg(&path)
            .status()?
    } else {
        std::process::Command::new("ffmpeg")
            .args(["-nostdin", "-v", "error", "-f", "x11grab", "-video_size"])
            .arg(format!("{}x{}", width * root_scale, height * root_scale))
            .args(["-i", &std::env::var("DISPLAY")?, "-frames:v", "1", "-y"])
            .arg(&path)
            .status()?
    };
    ensure!(status.success(), "display capture failed");
    let image = gtk::gdk_pixbuf::Pixbuf::from_file(&path)?;
    let pixels = image.read_pixel_bytes();
    let pixel = |x: u32, y: u32| {
        let offset = y as usize * image.rowstride() as usize + x as usize * image.n_channels() as usize;
        (pixels[offset], pixels[offset + 1], pixels[offset + 2])
    };
    let mut bounds = (u32::MAX, u32::MAX, 0, 0);
    let mut counts = vec![0; colors.len()];
    for y in 0..image.height() as u32 {
        for x in 0..image.width() as u32 {
            if let Some(index) = colors.iter().position(|color| *color == pixel(x, y)) {
                counts[index] += 1;
                bounds = (bounds.0.min(x), bounds.1.min(y), bounds.2.max(x), bounds.3.max(y));
            }
        }
    }
    let content = if u64::from(viewport.0) * u64::from(source.1) <= u64::from(viewport.1) * u64::from(source.0) {
        (
            viewport.0,
            (u64::from(viewport.0) * u64::from(source.1) / u64::from(source.0)).max(1) as u32,
        )
    } else {
        (
            (u64::from(viewport.1) * u64::from(source.0) / u64::from(source.1)).max(1) as u32,
            viewport.1,
        )
    };
    ensure!(counts.iter().all(|count| *count > 0), "{stage}: missing source color, counts={counts:?}");
    ensure!(
        (bounds.2 - bounds.0 + 1, bounds.3 - bounds.1 + 1) == content,
        "{stage}: color bounds {bounds:?} do not match expected content {content:?} in viewport {viewport:?} at scale {scale}"
    );
    ensure!(
        counts.iter().sum::<u32>() == content.0 * content.1,
        "{stage}: content contains unexpected pixels, counts={counts:?}"
    );
    if colors.len() == 2 {
        ensure!(counts[0] == counts[1], "{stage}: unequal nearest-filter color halves: {counts:?}");
        ensure!(pixel(bounds.0, bounds.1) == colors[0], "{stage}: left source color is incorrect");
        ensure!(pixel(bounds.2, bounds.1) == colors[1], "{stage}: right source color is incorrect");
    }
    let offset = ((viewport.0 - content.0) / 2, (viewport.1 - content.1) / 2);
    ensure!(bounds.0 >= offset.0 && bounds.1 >= offset.1, "{stage}: output is clipped above or left");
    let origin = (bounds.0 - offset.0, bounds.1 - offset.1);
    ensure!(
        origin.0 + viewport.0 <= image.width() as u32 && origin.1 + viewport.1 <= image.height() as u32,
        "{stage}: viewport extends beyond the screenshot"
    );
    for y in 0..viewport.1 {
        for x in 0..viewport.0 {
            if x < offset.0 || x >= offset.0 + content.0 || y < offset.1 || y >= offset.1 + content.1 {
                ensure!(
                    pixel(origin.0 + x, origin.1 + y) == (0, 0, 0),
                    "{stage}: letterbox is not black at ({x},{y})"
                );
            }
        }
    }
    println!("NATIVE_PIXELS {stage} viewport={viewport:?} source={source:?} scale={scale} bounds={bounds:?} counts={counts:?}");
    Ok(())
}
