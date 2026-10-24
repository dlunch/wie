use std::sync::Arc;

use anyhow::{Context, Error, Result, anyhow};
use wgpu::{Adapter, Device, DeviceDescriptor, Instance, Limits, Queue, RequestAdapterOptions, Surface};

pub struct GpuContext {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
}

impl GpuContext {
    /// The host must stop accepting this session's errors before dropping its resources.
    pub async fn new(instance: &Instance, surface: &Surface<'_>, on_error: impl Fn(Error) + Send + Sync + 'static) -> Result<Self> {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(surface),
                ..Default::default()
            })
            .await
            .context("No compatible presentation adapter")?;
        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("wie renderer"),
                required_limits: Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                ..Default::default()
            })
            .await
            .context("Failed to create presentation device")?;
        let on_error = Arc::new(on_error);
        let on_uncaptured = on_error.clone();
        device.on_uncaptured_error(Arc::new(move |error| on_uncaptured(anyhow!("GPU error: {error}"))));
        device.set_device_lost_callback(move |reason, message| on_error(anyhow!("GPU device lost ({reason:?}): {message}")));
        Ok(Self { adapter, device, queue })
    }
}
