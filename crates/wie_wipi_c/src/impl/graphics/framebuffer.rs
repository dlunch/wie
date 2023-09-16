use alloc::vec::Vec;
use core::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

use wie_backend::{Canvas, Image};

use crate::base::{CContext, CMemoryId};


/// MC_GrpDisplayInfo
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICDisplayInfo {
    pub bpp: u32,
    pub depth: u32,
    pub width: u32,
    pub height: u32,
    pub bpl: u32,
    pub color_type: u32,
    pub red_mask: u32,
    pub blue_mask: u32,
    pub green_mask: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICFramebuffer {
    pub width: u32,
    pub height: u32,
    bpl: u32,
    bpp: u32,
    buf: CMemoryId,
}

impl WIPICFramebuffer {
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
        let canvas = context.backend().screen_canvas();

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

    pub fn from_image(context: &mut dyn CContext, image: &Image) -> anyhow::Result<Self> {
        let buf = context.alloc(image.width() * image.height() * image.bytes_per_pixel())?;

        context.write_bytes(context.data_ptr(buf)?, image.raw_rgba())?;

        Ok(Self {
            width: image.width(),
            height: image.height(),
            bpl: image.width() * image.bytes_per_pixel(),
            bpp: image.bytes_per_pixel() * 8,
            buf,
        })
    }

    pub fn data(&self, context: &dyn CContext) -> anyhow::Result<Vec<u8>> {
        let data = context.read_bytes(context.data_ptr(self.buf)?, self.width * self.height * self.bpp / 8)?;

        Ok(data)
    }

    pub fn image(&self, context: &mut dyn CContext) -> anyhow::Result<Image> {
        Ok(Image::from_raw(self.width, self.height, self.data(context)?))
    }

    pub fn canvas<'a>(&'a self, context: &'a mut dyn CContext) -> anyhow::Result<FramebufferCanvas<'a>> {
        let canvas = Canvas::from_image(self.image(context)?);

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
    framebuffer: &'a WIPICFramebuffer,
    context: &'a mut dyn CContext,
    canvas: Canvas,
}

impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        self.framebuffer.write(self.context, self.canvas.raw_rgba()).unwrap()
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
