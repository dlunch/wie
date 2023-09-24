use std::{io::Cursor, ops::Deref};

use image::{imageops, io::Reader as ImageReader, GenericImageView, Rgba, RgbaImage};
use imageproc::{drawing::draw_filled_rect_mut, rect::Rect};

#[repr(C, packed)]
#[derive(Copy, Clone, Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn to_rgb32(self) -> u32 {
        ((self.r as u32) << 16) | ((self.g as u32) << 8) | (self.b as u32)
    }

    pub fn from_rgb32(v: u32, alpha: u8) -> Self {
        let a: u8 = alpha;
        let r = ((v >> 16) & 0xff) as u8;
        let g = ((v >> 8) & 0xff) as u8;
        let b = (v & 0xff) as u8;
        Self::new(r, g, b, a)
    }
}

impl From<Color> for Rgba<u8> {
    fn from(c: Color) -> Self {
        Self([c.r, c.g, c.b, c.a])
    }
}

pub struct Image {
    image: RgbaImage,
}

impl Image {
    pub fn from_raw(width: u32, height: u32, buf: Vec<u8>) -> Self {
        let image = RgbaImage::from_raw(width, height, buf).unwrap();

        Self { image }
    }

    pub fn from_size(width: u32, height: u32) -> Self {
        let image = RgbaImage::new(width, height);

        Self { image }
    }

    pub fn from_image(image: &[u8]) -> anyhow::Result<Self> {
        let image = ImageReader::new(Cursor::new(image)).with_guessed_format()?.decode()?;

        Ok(Self { image: image.into_rgba8() })
    }

    pub fn width(&self) -> u32 {
        self.image.width()
    }

    pub fn height(&self) -> u32 {
        self.image.height()
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        4
    }

    pub fn raw_rgba(&self) -> &[u8] {
        self.image.as_raw()
    }
}

pub struct Canvas {
    image: Image,
}

impl Canvas {
    pub fn from_image(image: Image) -> Self {
        Self { image }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &Image, sx: u32, sy: u32) {
        let src_view = src.image.view(sx, sy, w, h);

        imageops::overlay(&mut self.image.image, src_view.deref(), dx as i64, dy as i64);
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        let rect = Rect::at(x as i32, y as i32).of_size(w, h);
        let color = image::Rgba([color.r, color.g, color.b, color.a]);

        draw_filled_rect_mut(&mut self.image.image, rect, color);
    }

    pub fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.image.image.put_pixel(x, y, color.into());
    }
}

impl Deref for Canvas {
    type Target = Image;

    fn deref(&self) -> &Self::Target {
        &self.image
    }
}
