use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

use wie_backend::{Canvas, CanvasMut};

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

        context.write_bytes(context.data_ptr(buf)?, canvas.buffer())?;

        Ok(Self {
            width: canvas.width(),
            height: canvas.height(),
            bpl: canvas.width() * canvas.bytes_per_pixel(),
            bpp: canvas.bytes_per_pixel() * 8,
            buf,
        })
    }

    pub fn data(&self, context: &dyn CContext) -> anyhow::Result<Vec<u8>> {
        let data = context.read_bytes(context.data_ptr(self.buf)?, self.width * self.height * self.bpp / 8)?;

        Ok(data)
    }

    pub fn canvas(&self, context: &mut dyn CContext) -> anyhow::Result<Canvas> {
        Ok(Canvas::from_raw(self.width, self.height, self.data(context)?))
    }

    pub fn canvas_mut<'a>(&'a self, context: &'a mut dyn CContext) -> anyhow::Result<FramebufferCanvas<'a>> {
        let canvas = CanvasMut::from_raw(self.width, self.height, self.data(context)?);

        Ok(FramebufferCanvas {
            framebuffer: self,
            context,
            canvas,
        })
    }

    pub fn write(&self, context: &mut dyn CContext, data: &[u8]) -> anyhow::Result<()> {
        context.write_bytes(context.data_ptr(self.buf)?, data)
    }
}

pub struct FramebufferCanvas<'a> {
    framebuffer: &'a Framebuffer,
    context: &'a mut dyn CContext,
    canvas: CanvasMut,
}

impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        self.framebuffer.write(self.context, self.canvas.buffer()).unwrap()
    }
}

impl Deref for FramebufferCanvas<'_> {
    type Target = CanvasMut;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for FramebufferCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}
