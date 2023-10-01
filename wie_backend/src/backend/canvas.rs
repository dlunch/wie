use core::mem::size_of;
use std::io::Cursor;

use bytemuck::{cast_slice, pod_collect_to_vec, Pod};
use image::io::Reader as ImageReader;

#[derive(Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Color;
    fn raw(&self) -> &[u8];
    fn colors(&self) -> Vec<Color>;
}

pub trait Canvas: Image {
    #[allow(clippy::too_many_arguments)]
    fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &dyn Image, sx: u32, sy: u32);
    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color);
    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    fn put_pixel(&mut self, x: u32, y: u32, color: Color);
    fn image(self: Box<Self>) -> Box<dyn Image>;
}

pub trait PixelType {
    type DataType: Copy + Pod;
    fn from_color(color: Color) -> Self::DataType;
    fn to_color(raw: Self::DataType) -> Color;
    fn default() -> Self::DataType;
}

pub struct Rgb565Pixel {}

impl PixelType for Rgb565Pixel {
    type DataType = u16;

    fn from_color(color: Color) -> Self::DataType {
        let r = (color.r as u16) >> 3;
        let g = (color.g as u16) >> 2;
        let b = (color.b as u16) >> 3;

        (r << 11) | (g << 5) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 11) & 0x1f) as u8;
        let g = ((raw >> 5) & 0x3f) as u8;
        let b = (raw & 0x1f) as u8;

        let r = ((r as u32 * 255 + 15) / 31) as u8;
        let g = ((g as u32 * 255 + 31) / 63) as u8;
        let b = ((b as u32 * 255 + 15) / 31) as u8;

        Color { a: 0xff, r, g, b }
    }

    fn default() -> Self::DataType {
        0
    }
}

pub struct Rgb8Pixel {}

impl PixelType for Rgb8Pixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a: 0xff, r, g, b }
    }

    fn default() -> Self::DataType {
        0
    }
}

pub struct ArgbPixel {}

impl PixelType for ArgbPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        (color.a as u32) << 24 | (color.r as u32) << 16 | (color.g as u32) << 8 | color.b as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let r = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let b = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }

    fn default() -> Self::DataType {
        0
    }
}

pub struct ImageBuffer<T>
where
    T: PixelType,
{
    width: u32,
    height: u32,
    data: Vec<T::DataType>,
}

impl<T> ImageBuffer<T>
where
    T: PixelType,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![T::default(); (width * height) as usize],
        }
    }

    pub fn from_raw(width: u32, height: u32, raw: Vec<T::DataType>) -> Self {
        Self { width, height, data: raw }
    }
}

impl<T> Image for ImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn bytes_per_pixel(&self) -> u32 {
        size_of::<T::DataType>() as u32
    }

    fn get_pixel(&self, x: u32, y: u32) -> Color {
        let raw = self.data[(y * self.width + x) as usize];

        T::to_color(raw)
    }

    fn raw(&self) -> &[u8] {
        cast_slice(&self.data)
    }

    fn colors(&self) -> Vec<Color> {
        self.data.iter().map(|&x| T::to_color(x)).collect()
    }
}

impl<T> Canvas for ImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &dyn Image, sx: u32, sy: u32) {
        for y in 0..h {
            for x in 0..w {
                if sx + x >= src.width() || sy + y >= src.height() {
                    continue;
                }
                if dx + x >= self.width || dy + y >= self.height {
                    continue;
                }

                let color = src.get_pixel(sx + x, sy + y);

                if color.a == 0 {
                    continue; // TODO alpha blending
                }

                self.put_pixel(dx + x, dy + y, color);
            }
        }
    }

    // TODO change it to bresenham's or something..
    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color) {
        let dx = (x2 as f32) - (x1 as f32);
        let dy = (y2 as f32) - (y1 as f32);

        let mut x = x1 as f32;
        let mut y = y1 as f32;

        let step = dx.abs().max(dy.abs());

        let dx = dx / step;
        let dy = dy / step;

        for _ in 0..step as u32 {
            if x >= self.width as f32 || y >= self.height as f32 {
                continue;
            }
            self.put_pixel(x as u32, y as u32, color);

            x += dx;
            y += dy;
        }
    }

    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for x in x..x + w {
            if x >= self.width {
                continue;
            }
            self.put_pixel(x, y, color);
            self.put_pixel(x, y + h - 1, color);
        }
        for y in y..y + h {
            if y >= self.height {
                continue;
            }
            self.put_pixel(x, y, color);
            self.put_pixel(x + w - 1, y, color);
        }
    }

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for y in y..y + h {
            for x in x..x + w {
                if x >= self.width || y >= self.height {
                    continue;
                }
                self.put_pixel(x, y, color);
            }
        }
    }

    fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        let raw = T::from_color(color);

        self.data[(y * self.width + x) as usize] = raw;
    }

    fn image(self: Box<Self>) -> Box<dyn Image> {
        self
    }
}

pub fn decode_image(data: &[u8]) -> anyhow::Result<Box<dyn Image>> {
    let image = ImageReader::new(Cursor::new(&data)).with_guessed_format()?.decode()?;
    let rgba = image.into_rgba8();

    let data = rgba.pixels().flat_map(|x| [x.0[2], x.0[1], x.0[0], x.0[3]]).collect::<Vec<_>>();

    Ok(create_canvas(rgba.width(), rgba.height(), 32, &data)?.image())
}

pub fn create_canvas(width: u32, height: u32, bpp: u32, data: &[u8]) -> anyhow::Result<Box<dyn Canvas>> {
    Ok(match bpp {
        16 => Box::new(ImageBuffer::<Rgb565Pixel>::from_raw(width, height, pod_collect_to_vec(data))),
        32 => Box::new(ImageBuffer::<ArgbPixel>::from_raw(width, height, pod_collect_to_vec(data))), // TODO we can remove copy
        _ => anyhow::bail!("Unsupported bpp {}", bpp),
    })
}
