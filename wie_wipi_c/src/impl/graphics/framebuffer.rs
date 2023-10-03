use alloc::{boxed::Box, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::{Pod, Zeroable};

use wie_backend::canvas::{create_canvas, Canvas, Image, PixelFormat};

use crate::base::{CContext, CMemoryId, CWord};

/// MC_GrpDisplayInfo
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICDisplayInfo {
    pub bpp: CWord,
    pub depth: CWord,
    pub width: CWord,
    pub height: CWord,
    pub bpl: CWord,
    pub color_type: CWord,
    pub red_mask: CWord,
    pub blue_mask: CWord,
    pub green_mask: CWord,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICFramebuffer {
    pub width: CWord,
    pub height: CWord,
    pub bpl: CWord,
    pub bpp: CWord,
    pub buf: CMemoryId,
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

    pub fn new(context: &mut dyn CContext, width: CWord, height: CWord, bpp: CWord) -> anyhow::Result<Self> {
        let bytes_per_pixel = bpp / 8;

        let buf = context.alloc(width * height * bytes_per_pixel)?;

        Ok(Self {
            width,
            height,
            bpl: width * bytes_per_pixel,
            bpp: bytes_per_pixel * 8,
            buf,
        })
    }

    pub fn from_image(context: &mut dyn CContext, image: &dyn Image) -> anyhow::Result<Self> {
        let buf = context.alloc(image.width() * image.height() * image.bytes_per_pixel())?;

        context.write_bytes(context.data_ptr(buf)?, image.raw())?;

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

    pub fn image(&self, context: &mut dyn CContext) -> anyhow::Result<Box<dyn Image>> {
        let data = self.data(context)?;

        Ok(create_canvas(self.width, self.height, self.pixel_format(), &data)?.image())
    }

    pub fn canvas<'a>(&'a self, context: &'a mut dyn CContext) -> anyhow::Result<FramebufferCanvas<'a>> {
        let data = self.data(context)?;

        let canvas = create_canvas(self.width, self.height, self.pixel_format(), &data)?;

        Ok(FramebufferCanvas {
            framebuffer: self,
            context,
            canvas,
        })
    }

    pub fn write(&self, context: &mut dyn CContext, data: &[u8]) -> anyhow::Result<()> {
        context.write_bytes(context.data_ptr(self.buf)?, data)
    }

    fn pixel_format(&self) -> PixelFormat {
        match self.bpp {
            16 => PixelFormat::Rgb565,
            32 => PixelFormat::Argb,
            _ => panic!("Unsupported pixel format: {}", self.bpp),
        }
    }
}

pub struct FramebufferCanvas<'a> {
    framebuffer: &'a WIPICFramebuffer,
    context: &'a mut dyn CContext,
    canvas: Box<dyn Canvas>,
}

impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        self.framebuffer.write(self.context, self.canvas.raw()).unwrap()
    }
}

impl Deref for FramebufferCanvas<'_> {
    type Target = Box<dyn Canvas>;

    fn deref(&self) -> &Self::Target {
        &self.canvas
    }
}

impl DerefMut for FramebufferCanvas<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.canvas
    }
}
