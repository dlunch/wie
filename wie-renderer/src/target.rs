use anyhow::{Result, ensure};
use wgpu::{Extent3d, TexelCopyBufferLayout, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};

use crate::{Frame, GpuContext};

/// Source-sized storage shared by CPU uploads and ordered GPU producers.
pub struct RenderTarget {
    pub texture: Texture,
    pub view: TextureView,
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(context: &GpuContext, width: u32, height: u32) -> Result<Self> {
        let limit = context.device.limits().max_texture_dimension_2d;
        ensure!(
            width > 0 && height > 0 && width <= limit && height <= limit,
            "Unsupported source dimensions {width}x{height} (device limit {limit})"
        );
        let texture = context.device.create_texture(&TextureDescriptor {
            label: Some("wie source"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        Ok(Self {
            texture,
            view,
            width,
            height,
        })
    }

    /// Upload a frame with this target's source dimensions before presenting it.
    pub fn upload(&self, context: &GpuContext, frame: &Frame) {
        context.queue.write_texture(
            self.texture.as_image_copy(),
            &frame.rgba,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(frame.width * 4),
                rows_per_image: None,
            },
            self.texture.size(),
        );
    }
}
