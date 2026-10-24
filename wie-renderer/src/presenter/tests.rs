use std::sync::mpsc;

use futures::executor::block_on;
use wgpu::{
    BufferDescriptor, BufferUsages, Color, Extent3d, FilterMode, MapMode, PollType, TexelCopyBufferInfo, TexelCopyBufferLayout, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages,
};

use super::PresentationPass;
use crate::{Frame, GpuContext, RenderTarget};

fn pixels(
    context: &GpuContext,
    pass: &mut PresentationPass,
    target: Option<&RenderTarget>,
    format: TextureFormat,
    width: u32,
    height: u32,
) -> Vec<[u8; 4]> {
    let output = context.device.create_texture(&TextureDescriptor {
        label: Some("test output"),
        size: Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let row_bytes = (width * 4).div_ceil(256) * 256;
    let readback = context.device.create_buffer(&BufferDescriptor {
        label: Some("test readback"),
        size: u64::from(row_bytes) * u64::from(height),
        usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = context.device.create_command_encoder(&Default::default());
    pass.encode(context, &mut encoder, &output.create_view(&Default::default()), target, width, height);
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        TexelCopyBufferInfo {
            buffer: &readback,
            layout: TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: None,
            },
        },
        output.size(),
    );
    context.queue.submit([encoder.finish()]);
    let (sender, receiver) = mpsc::channel();
    readback.slice(..).map_async(MapMode::Read, move |result| sender.send(result).unwrap());
    context.device.poll(PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = readback.slice(..).get_mapped_range().unwrap();
    mapped
        .chunks(row_bytes as usize)
        .flat_map(|row| {
            row[..width as usize * 4].chunks_exact(4).map(|pixel| {
                if matches!(format, TextureFormat::Bgra8Unorm | TextureFormat::Bgra8UnormSrgb) {
                    [pixel[2], pixel[1], pixel[0], pixel[3]]
                } else {
                    pixel.try_into().unwrap()
                }
            })
        })
        .collect()
}

fn near(actual: [u8; 4], expected: [u8; 4]) {
    assert!(actual.iter().zip(expected).all(|(a, b)| a.abs_diff(b) <= 1), "{actual:?} != {expected:?}");
}

#[test]
#[ignore = "requires a native GPU adapter; run explicitly with --ignored"]
fn presents_cpu_and_gpu_content_with_srgb_filtering_and_retention() {
    let context = block_on(async {
        let instance = wgpu::Instance::default();
        let adapter = instance.request_adapter(&Default::default()).await.expect("GPU test requires an adapter");
        eprintln!("Renderer test adapter: {:?}", adapter.get_info());
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .unwrap();
        GpuContext { adapter, device, queue }
    });
    assert!(RenderTarget::new(&context, 0, 2).is_err());
    assert!(RenderTarget::new(&context, context.device.limits().max_texture_dimension_2d + 1, 1).is_err());
    let frame = Frame {
        width: 3,
        height: 2,
        rgba: vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 64, 64, 64, 255, 128, 128, 128, 255, 255, 255, 255, 255,
        ],
    };
    let target = RenderTarget::new(&context, frame.width, frame.height).unwrap();
    target.upload(&context, &frame);
    let transition = RenderTarget::new(&context, 2, 1).unwrap();
    transition.upload(
        &context,
        &Frame {
            width: 2,
            height: 1,
            rgba: vec![0, 0, 0, 255, 255, 255, 255, 255],
        },
    );

    for format in [
        TextureFormat::Rgba8UnormSrgb,
        TextureFormat::Rgba8Unorm,
        TextureFormat::Bgra8UnormSrgb,
        TextureFormat::Bgra8Unorm,
    ] {
        for filter in [FilterMode::Nearest, FilterMode::Linear] {
            let mut pass = PresentationPass::new(&context.device, format, filter);
            assert_eq!(pixels(&context, &mut pass, None, format, 4, 4), vec![[0, 0, 0, 255]; 16]);
            let actual = pixels(&context, &mut pass, Some(&target), format, 3, 2);
            for (actual, expected) in actual.into_iter().zip(frame.rgba.chunks_exact(4)) {
                near(actual, expected.try_into().unwrap());
            }
            let actual = pixels(&context, &mut pass, Some(&target), format, 12, 4);
            assert_eq!(actual[0], [0, 0, 0, 255]);
            assert_eq!(actual[11], [0, 0, 0, 255]);
            near(actual[3], [255, 0, 0, 255]);
            near(actual[3 * 12 + 8], [255, 255, 255, 255]);
            let actual = pixels(&context, &mut pass, Some(&transition), format, 3, 3);
            if filter == FilterMode::Linear {
                near(actual[4], [188, 188, 188, 255]);
                near(pixels(&context, &mut pass, Some(&transition), format, 1, 1)[0], [188, 188, 188, 255]);
            } else {
                assert!(matches!(actual[4], [0, 0, 0, 255] | [255, 255, 255, 255]));
                assert!(matches!(
                    pixels(&context, &mut pass, Some(&transition), format, 1, 1)[0],
                    [0, 0, 0, 255] | [255, 255, 255, 255]
                ));
            }
        }
    }

    // A producer submits directly to the same target, without any CPU upload.
    let producer = RenderTarget::new(&context, 3, 2).unwrap();
    let mut encoder = context.device.create_command_encoder(&Default::default());
    {
        let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &producer.view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(Color {
                        r: 0.25,
                        g: 0.5,
                        b: 0.75,
                        a: 0.2,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
    }
    context.queue.submit([encoder.finish()]);
    for format in [TextureFormat::Rgba8UnormSrgb, TextureFormat::Rgba8Unorm] {
        let mut pass = PresentationPass::new(&context.device, format, FilterMode::Linear);
        for (width, height) in [(3, 2), (12, 12), (9, 6)] {
            let actual = pixels(&context, &mut pass, Some(&producer), format, width, height);
            near(actual[(height / 2 * width + width / 2) as usize], [137, 188, 225, 255]);
            if width == height {
                assert_eq!(actual[0], [0, 0, 0, 255]);
            }
        }
    }
}
