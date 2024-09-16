use alloc::{boxed::Box, vec, vec::Vec};
use core::ops::{Deref, DerefMut};

use bytemuck::{pod_collect_to_vec, Pod, Zeroable};

use wie_backend::canvas::{ArgbPixel, Canvas, Image, ImageBufferCanvas, Rgb565Pixel, VecImageBuffer};

use crate::{context::WIPICContext, WIPICMemoryId, WIPICResult, WIPICWord};

/// MC_GrpDisplayInfo
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICDisplayInfo {
    pub bpp: WIPICWord,
    pub depth: WIPICWord,
    pub width: WIPICWord,
    pub height: WIPICWord,
    pub bpl: WIPICWord,
    pub color_type: WIPICWord,
    pub red_mask: WIPICWord,
    pub blue_mask: WIPICWord,
    pub green_mask: WIPICWord,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct WIPICFramebuffer {
    pub width: WIPICWord,
    pub height: WIPICWord,
    pub bpl: WIPICWord,
    pub bpp: WIPICWord,
    pub buf: WIPICMemoryId,
}

impl WIPICFramebuffer {
    pub fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            bpl: 0,
            bpp: 0,
            buf: WIPICMemoryId(0),
        }
    }

    pub fn new(context: &mut dyn WIPICContext, width: WIPICWord, height: WIPICWord, bpp: WIPICWord) -> WIPICResult<Self> {
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

    pub fn from_image(context: &mut dyn WIPICContext, image: &dyn Image) -> WIPICResult<Self> {
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

    pub fn data(&self, context: &dyn WIPICContext) -> WIPICResult<Vec<u8>> {
        let size = self.width * self.height * self.bpp / 8;
        let mut buf = vec![0; size as _];
        context.read_bytes(context.data_ptr(self.buf)?, &mut buf)?;

        Ok(buf)
    }

    pub fn image(&self, context: &mut dyn WIPICContext) -> WIPICResult<Box<dyn Image>> {
        let data = self.data(context)?;

        Ok(match self.bpp {
            16 => Box::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
                self.width as _,
                self.height as _,
                pod_collect_to_vec(&data),
            )),
            32 => Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
                self.width as _,
                self.height as _,
                pod_collect_to_vec(&data),
            )),
            _ => unimplemented!("Unsupported pixel format: {}", self.bpp),
        })
    }

    pub fn canvas<'a>(&'a self, context: &'a mut dyn WIPICContext) -> WIPICResult<FramebufferCanvas<'a>> {
        let data = self.data(context)?;

        let canvas: Box<dyn Canvas> = match self.bpp {
            16 => Box::new(ImageBufferCanvas::new(VecImageBuffer::<Rgb565Pixel>::from_raw(
                self.width as _,
                self.height as _,
                pod_collect_to_vec(&data),
            ))),
            32 => Box::new(ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::from_raw(
                self.width as _,
                self.height as _,
                pod_collect_to_vec(&data),
            ))),
            _ => unimplemented!("Unsupported pixel format: {}", self.bpp),
        };

        Ok(FramebufferCanvas {
            framebuffer: self,
            context,
            canvas,
        })
    }

    pub fn write(&self, context: &mut dyn WIPICContext, data: &[u8]) -> WIPICResult<()> {
        Ok(context.write_bytes(context.data_ptr(self.buf)?, data)?)
    }
}

pub struct FramebufferCanvas<'a> {
    framebuffer: &'a WIPICFramebuffer,
    context: &'a mut dyn WIPICContext,
    canvas: Box<dyn Canvas>,
}

impl Drop for FramebufferCanvas<'_> {
    fn drop(&mut self) {
        self.framebuffer.write(self.context, self.canvas.image().raw()).unwrap()
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
