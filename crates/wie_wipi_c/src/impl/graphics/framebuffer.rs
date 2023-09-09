use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use bytemuck::{cast_slice, cast_vec, Pod, Zeroable};

use wie_backend::Canvas;

use crate::base::{CContext, CMemoryId};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Framebuffer {
    pub width: u32,
    pub height: u32,
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

    pub fn from_screen_canvas(context: &mut dyn CContext) -> anyhow::Result<Self> {
        let canvas = context.backend().screen_canvas_mut();

        let (width, height, bytes_per_pixel) = (canvas.width(), canvas.height(), canvas.bytes_per_pixel());
        core::mem::drop(canvas);

        let buf = context.alloc(width * height * bytes_per_pixel)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn from_canvas(context: &mut dyn CContext, canvas: &Canvas) -> anyhow::Result<Self> {
        let buf = context.alloc(canvas.width() * canvas.height() * canvas.bytes_per_pixel())?;

        let canvas_buf = canvas.buffer().to_vec();
        let data = cast_slice(&canvas_buf);
        context.write_bytes(context.data_ptr(buf)?, data)?;

        Ok(Self {
            width: canvas.width(),
            height: canvas.height(),
            bpl: canvas.width() * canvas.bytes_per_pixel(),
            bpp: canvas.bytes_per_pixel() * 8,
            buf,
        })
    }

    pub fn data(&self, context: &dyn CContext) -> anyhow::Result<Vec<u32>> {
        let raw = context.read_bytes(context.data_ptr(self.buf)?, self.width * self.height * self.bpp / 8)?;
        let data = cast_vec(raw);

        Ok(data)
    }

    pub fn canvas<'a>(&'a self, context: &'a mut dyn CContext) -> anyhow::Result<FramebufferCanvas<'a>> {
        let canvas = Canvas::from_raw(self.width, self.height, self.data(context)?);

        Ok(FramebufferCanvas {
            framebuffer: self,
            context,
            canvas,
        })
    }

    pub fn write(&self, context: &mut dyn CContext, data: &[u32]) -> anyhow::Result<()> {
        let data = cast_slice(data);

        context.write_bytes(context.data_ptr(self.buf)?, data)
    }
}

pub struct FramebufferCanvas<'a> {
    framebuffer: &'a Framebuffer,
    context: &'a mut dyn CContext,
    canvas: Canvas,
}

impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        self.framebuffer.write(self.context, self.canvas.buffer()).unwrap()
    }
}

impl Deref for FramebufferCanvas<'_> {
    type Target = Canvas;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for FramebufferCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}
