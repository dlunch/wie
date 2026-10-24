use anyhow::{Context, Result, bail, ensure};
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindingResource, Color, ColorTargetState, ColorWrites, CommandEncoder, CompositeAlphaMode,
    CurrentSurfaceTexture, Device, FilterMode, FragmentState, LoadOp, Operations, RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline,
    RenderPipelineDescriptor, Sampler, SamplerDescriptor, ShaderModuleDescriptor, ShaderSource, StoreOp, Surface, SurfaceConfiguration,
    TextureFormat, TextureView, VertexState,
};

use crate::{GpuContext, RenderTarget};

pub enum Presentation {
    Presented,
    Retry,
    WaitForOutput,
    RecreateSurface,
}

pub struct Presenter<'window> {
    surface: Surface<'window>,
    config: SurfaceConfiguration,
    pass: PresentationPass,
    reconfigure: bool,
}

impl<'window> Presenter<'window> {
    /// Starts unconfigured; the host supplies physical output dimensions with `resize`.
    pub fn new(context: &GpuContext, surface: Surface<'window>, filter: FilterMode) -> Result<Self> {
        let mut config = surface
            .get_default_config(&context.adapter, 0, 0)
            .context("Surface is incompatible with the presentation adapter")?;
        let capabilities = surface.get_capabilities(&context.adapter);
        config.format = capabilities
            .formats
            .iter()
            .copied()
            .find(TextureFormat::is_srgb)
            .or_else(|| {
                capabilities
                    .formats
                    .iter()
                    .copied()
                    .find(|format| matches!(format, TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm))
            })
            .context("Surface has no supported SDR color format")?;
        config.alpha_mode = capabilities
            .alpha_modes
            .iter()
            .copied()
            .find(|mode| *mode == CompositeAlphaMode::Opaque)
            .or_else(|| capabilities.alpha_modes.first().copied())
            .context("Surface has no supported alpha mode")?;
        let pass = PresentationPass::new(&context.device, config.format, filter);
        Ok(Self {
            surface,
            config,
            pass,
            reconfigure: false,
        })
    }

    pub fn resize(&mut self, context: &GpuContext, width: u32, height: u32) -> Result<()> {
        let limit = context.device.limits().max_texture_dimension_2d;
        ensure!(
            width <= limit && height <= limit,
            "Unsupported output dimensions {width}x{height} (device limit {limit})"
        );
        self.config.width = width;
        self.config.height = height;
        if width > 0 && height > 0 {
            self.surface.configure(&context.device, &self.config);
            self.reconfigure = false;
        }
        Ok(())
    }

    /// Samples initialized content only. Producers must submit to this context's queue first.
    pub fn present(&mut self, context: &GpuContext, target: Option<&RenderTarget>) -> Result<Presentation> {
        if self.config.width == 0 || self.config.height == 0 {
            return Ok(Presentation::WaitForOutput);
        }
        if self.reconfigure {
            self.surface.configure(&context.device, &self.config);
            self.reconfigure = false;
        }
        let output = match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(output) => output,
            CurrentSurfaceTexture::Suboptimal(output) => {
                self.reconfigure = true;
                output
            }
            CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&context.device, &self.config);
                return Ok(Presentation::Retry);
            }
            CurrentSurfaceTexture::Timeout => return Ok(Presentation::Retry),
            CurrentSurfaceTexture::Occluded => return Ok(Presentation::WaitForOutput),
            CurrentSurfaceTexture::Lost => return Ok(Presentation::RecreateSurface),
            CurrentSurfaceTexture::Validation => bail!("Presentation surface validation failed"),
        };
        let view = output.texture.create_view(&Default::default());
        let mut encoder = context.device.create_command_encoder(&Default::default());
        self.pass
            .encode(context, &mut encoder, &view, target, self.config.width, self.config.height);
        context.queue.submit([encoder.finish()]);
        context.queue.present(output);
        Ok(Presentation::Presented)
    }
}

struct PresentationPass {
    pipeline: RenderPipeline,
    sampler: Sampler,
    binding: Option<(TextureView, BindGroup)>,
}

impl PresentationPass {
    fn new(device: &Device, format: TextureFormat, filter: FilterMode) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("wie presentation"),
            source: ShaderSource::Wgsl(include_str!("presentation.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("wie presentation"),
            layout: None,
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some(if format.is_srgb() { "fragment_srgb" } else { "fragment_unorm" }),
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("wie scale filter"),
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        });
        Self {
            pipeline,
            sampler,
            binding: None,
        }
    }

    fn encode(
        &mut self,
        context: &GpuContext,
        encoder: &mut CommandEncoder,
        output: &TextureView,
        target: Option<&RenderTarget>,
        width: u32,
        height: u32,
    ) {
        if let Some(target) = target
            && self.binding.as_ref().is_none_or(|(view, _)| *view != target.view)
        {
            let binding = context.device.create_bind_group(&BindGroupDescriptor {
                label: Some("wie source binding"),
                layout: &self.pipeline.get_bind_group_layout(0),
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: BindingResource::TextureView(&target.view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: BindingResource::Sampler(&self.sampler),
                    },
                ],
            });
            self.binding = Some((target.view.clone(), binding));
        }
        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: Some("wie presentation"),
            color_attachments: &[Some(RenderPassColorAttachment {
                view: output,
                depth_slice: None,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        if let Some(target) = target
            && let Some((_, binding)) = &self.binding
        {
            // GLES truncates fractional viewports, including subpixel dimensions to zero.
            let (scaled_width, scaled_height) = if u64::from(width) * u64::from(target.height) <= u64::from(height) * u64::from(target.width) {
                (
                    width,
                    (u64::from(target.height) * u64::from(width) / u64::from(target.width)).max(1) as u32,
                )
            } else {
                (
                    (u64::from(target.width) * u64::from(height) / u64::from(target.height)).max(1) as u32,
                    height,
                )
            };
            pass.set_viewport(
                ((width - scaled_width) / 2) as f32,
                ((height - scaled_height) / 2) as f32,
                scaled_width as f32,
                scaled_height as f32,
                0.0,
                1.0,
            );
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, binding, &[]);
            pass.draw(0..3, 0..1);
        }
    }
}
