//! Browser-side presentation probe; built as a WASM cdylib.

#[cfg(target_arch = "wasm32")]
mod probe {
    use std::sync::{Arc, Mutex};

    use wasm_bindgen::prelude::*;
    use web_sys::HtmlCanvasElement;
    use wgpu::{Color, FilterMode, InstanceDescriptor, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceTarget};

    use wie_backend::canvas::{ArgbPixel, VecImageBuffer};
    use wie_renderer::{Frame, GpuContext, Presentation, Presenter, RenderTarget};

    #[wasm_bindgen]
    pub struct BrowserProbe {
        presenter: Presenter<'static>,
        target: Option<RenderTarget>,
        context: GpuContext,
        canvas: HtmlCanvasElement,
        errors: Arc<Mutex<Vec<String>>>,
    }

    #[wasm_bindgen]
    impl BrowserProbe {
        pub async fn create(canvas: HtmlCanvasElement, nearest: bool) -> Result<BrowserProbe, JsError> {
            let instance = wgpu::util::new_instance_with_webgpu_detection(InstanceDescriptor::new_without_display_handle()).await;
            let surface = instance.create_surface(SurfaceTarget::Canvas(canvas.clone()))?;
            let errors = Arc::new(Mutex::new(Vec::new()));
            let sink = errors.clone();
            let context = GpuContext::new(&instance, &surface, move |error| sink.lock().unwrap().push(error.to_string()))
                .await
                .map_err(|error| JsError::new(&format!("{error:#}")))?;
            let mut presenter = Presenter::new(&context, surface, if nearest { FilterMode::Nearest } else { FilterMode::Linear })
                .map_err(|error| JsError::new(&format!("{error:#}")))?;
            presenter
                .resize(&context, canvas.width(), canvas.height())
                .map_err(|error| JsError::new(&format!("{error:#}")))?;
            Ok(Self {
                presenter,
                target: None,
                context,
                canvas,
                errors,
            })
        }

        pub fn adapter(&self) -> String {
            format!("{:?}", self.context.adapter.get_info())
        }

        pub fn errors(&self) -> Vec<String> {
            self.errors.lock().unwrap().clone()
        }

        pub fn cpu(&mut self) -> Result<(), JsError> {
            let image = VecImageBuffer::<ArgbPixel>::from_raw(3, 2, vec![0xffff0000, 0xff00ff00, 0xff0000ff, 0x00404040, 0x80808080, 0xffffffff]);
            let frame = Frame::from_image(&image);
            let target = RenderTarget::new(&self.context, frame.width, frame.height).map_err(|error| JsError::new(&format!("{error:#}")))?;
            target.upload(&self.context, &frame);
            self.target = Some(target);
            Ok(())
        }

        pub fn transition(&mut self) -> Result<(), JsError> {
            let target = RenderTarget::new(&self.context, 2, 1).map_err(|error| JsError::new(&format!("{error:#}")))?;
            target.upload(
                &self.context,
                &Frame {
                    width: 2,
                    height: 1,
                    rgba: vec![0, 0, 0, 255, 255, 255, 255, 255],
                },
            );
            self.target = Some(target);
            Ok(())
        }

        pub fn gpu(&mut self) -> Result<(), JsError> {
            let target = RenderTarget::new(&self.context, 3, 2).map_err(|error| JsError::new(&format!("{error:#}")))?;
            let mut encoder = self.context.device.create_command_encoder(&Default::default());
            {
                let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &target.view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: 0.25,
                                g: 0.5,
                                b: 0.75,
                                a: 0.2,
                            }),
                            store: StoreOp::Store,
                        },
                    })],
                    ..Default::default()
                });
            }
            self.context.queue.submit([encoder.finish()]);
            self.target = Some(target);
            Ok(())
        }

        pub fn resize(&mut self, width: u32, height: u32) -> Result<(), JsError> {
            self.canvas.set_width(width);
            self.canvas.set_height(height);
            self.presenter
                .resize(&self.context, width, height)
                .map_err(|error| JsError::new(&format!("{error:#}")))?;
            Ok(())
        }

        pub fn present(&mut self) -> Result<String, JsError> {
            Ok(match self
                .presenter
                .present(&self.context, self.target.as_ref())
                .map_err(|error| JsError::new(&format!("{error:#}")))?
            {
                Presentation::Presented => "presented",
                Presentation::Retry => "retry",
                Presentation::WaitForOutput => "wait",
                Presentation::RecreateSurface => "recreate",
            }
            .into())
        }
    }
}
