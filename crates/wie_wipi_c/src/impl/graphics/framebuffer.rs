use alloc::{vec, vec::Vec};
use core::{mem::forget, slice};

use wie_backend::CanvasHandle;

use crate::base::{CContext, CMemoryId};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Framebuffer {
    width: u32,
    height: u32,
    bpl: u32,
    bpp: u32,
    buf: CMemoryId,
}

impl Framebuffer {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            bpl: 0,
            bpp: 0,
            buf: CMemoryId(0),
        }
    }

    pub fn from_compatible_canvas(context: &mut dyn CContext, canvas_handle: CanvasHandle) -> anyhow::Result<Self> {
        let (width, height, bytes_per_pixel) = {
            let mut canvases = context.backend().canvases_mut();
            let canvas = canvases.canvas(canvas_handle);

            (canvas.width(), canvas.height(), canvas.bytes_per_pixel())
        };

        let buf = context.alloc(width * height * bytes_per_pixel)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn from_canvas(context: &mut dyn CContext, canvas_handle: CanvasHandle) -> anyhow::Result<Self> {
        let (width, height, bytes_per_pixel, data) = {
            let mut canvases = context.backend().canvases_mut();
            let canvas = canvases.canvas(canvas_handle);

            let mut data = vec![0; (canvas.width() * canvas.height()) as usize];
            canvas.copy(&mut data);

            (canvas.width(), canvas.height(), canvas.bytes_per_pixel(), data)
        };

        let data = unsafe { slice::from_raw_parts(data.as_ptr() as _, data.len() * 4) }; // TODO
        let buf = context.alloc(width * height * bytes_per_pixel)?;
        context.write_bytes(context.data_ptr(buf)?, data)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn data(&self, context: &dyn CContext) -> anyhow::Result<Vec<u32>> {
        let mut raw = context.read_bytes(context.data_ptr(self.buf)?, self.width * self.height * self.bpp / 8)?;

        // TODO
        let data = unsafe { Vec::from_raw_parts(raw.as_mut_ptr() as _, raw.len() / 4, raw.capacity() / 4) };
        forget(raw);

        Ok(data)
    }
}
