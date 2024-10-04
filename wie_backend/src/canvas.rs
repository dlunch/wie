use core::mem::size_of;

use ab_glyph::{Font, FontRef, ScaleFont};
use bytemuck::{cast_slice, pod_collect_to_vec, Pod};
use image::ImageReader;
use num_traits::{Num, Zero};

use wie_util::{Result, WieError};

lazy_static::lazy_static! {
    static ref FONT: FontRef<'static> = FontRef::try_from_slice(include_bytes!("../../fonts/neodgm.ttf")).unwrap();
}

pub enum TextAlignment {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy)]
pub struct Color {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub trait Image: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn bytes_per_pixel(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> Color;
    fn raw(&self) -> &[u8];
    fn colors(&self) -> Vec<Color>;
}

pub trait ImageBuffer: Send {
    fn put_pixel(&mut self, x: u32, y: u32, color: Color);
    fn put_pixels(&mut self, x: u32, y: u32, width: u32, colors: &[Color]);
}

pub trait Canvas: Send {
    fn image(&self) -> &dyn Image;
    #[allow(clippy::too_many_arguments)]
    fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &dyn Image, sx: u32, sy: u32, clip: Clip);
    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color);
    fn draw_text(&mut self, string: &str, x: u32, y: u32, text_alignment: TextAlignment);
    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color);
    fn put_pixel(&mut self, x: u32, y: u32, color: Color);
}

pub trait PixelType: Send {
    type DataType: Copy + Pod + Num + Send;
    fn from_color(color: Color) -> Self::DataType;
    fn to_color(raw: Self::DataType) -> Color;
}

pub struct Rgb565Pixel;

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
}

pub struct Rgb8Pixel;

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
}

pub struct ArgbPixel;

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
}

pub struct AbgrPixel;

impl PixelType for AbgrPixel {
    type DataType = u32;

    fn from_color(color: Color) -> Self::DataType {
        (color.a as u32) << 24 | (color.b as u32) << 16 | (color.g as u32) << 8 | color.r as u32
    }

    fn to_color(raw: Self::DataType) -> Color {
        let a = ((raw >> 24) & 0xff) as u8;
        let b = ((raw >> 16) & 0xff) as u8;
        let g = ((raw >> 8) & 0xff) as u8;
        let r = (raw & 0xff) as u8;

        Color { a, r, g, b }
    }
}

pub struct VecImageBuffer<T>
where
    T: PixelType,
{
    width: u32,
    height: u32,
    data: Vec<T::DataType>,
}

impl<T> VecImageBuffer<T>
where
    T: PixelType,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![T::DataType::zero(); (width * height) as usize],
        }
    }

    pub fn from_raw(width: u32, height: u32, raw: Vec<T::DataType>) -> Self {
        Self { width, height, data: raw }
    }
}

impl<T> Image for VecImageBuffer<T>
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

impl<T> ImageBuffer for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let raw = T::from_color(color);

        self.data[(y * self.width + x) as usize] = raw;
    }

    fn put_pixels(&mut self, x: u32, y: u32, width: u32, colors: &[Color]) {
        for (i, color) in colors.iter().enumerate() {
            let x = x + (i as u32 % width);
            let y = y + (i as u32 / width);

            if x >= self.width || y >= self.height {
                continue;
            }

            let raw = T::from_color(*color);

            self.data[(y * self.width + x) as usize] = raw;
        }
    }
}

pub struct ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    image_buffer: T,
}

impl<T> ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    pub fn new(image_buffer: T) -> Self {
        Self { image_buffer }
    }

    pub fn into_inner(self) -> T {
        self.image_buffer
    }

    fn blend_pixel(&mut self, x: u32, y: u32, color: Color) {
        let bg = self.image_buffer.get_pixel(x, y);
        let factor = color.a as f32 / 255.0;

        let computed_color = Color {
            a: 0xff,
            r: (color.r as f32 * factor + bg.r as f32 * (1.0 - factor)) as u8,
            g: (color.g as f32 * factor + bg.g as f32 * (1.0 - factor)) as u8,
            b: (color.b as f32 * factor + bg.b as f32 * (1.0 - factor)) as u8,
        };

        self.put_pixel(x, y, computed_color);
    }
}

impl<T> Canvas for ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    fn image(&self) -> &dyn Image {
        &self.image_buffer
    }

    #[allow(clippy::too_many_arguments)]
    fn draw(&mut self, dx: u32, dy: u32, w: u32, h: u32, src: &dyn Image, sx: u32, sy: u32, clip: Clip) {
        for y in 0..h {
            for x in 0..w {
                if sx + x >= src.width() || sy + y >= src.height() {
                    continue;
                }
                if dx + x >= self.image_buffer.width() || dy + y >= self.image_buffer.height() {
                    continue;
                }
                if dx + x < clip.x || dx + x >= clip.x + clip.width || dy + y < clip.y || dy + y >= clip.y + clip.height {
                    continue;
                }

                self.blend_pixel(dx + x, dy + y, src.get_pixel(sx + x, sy + y));
            }
        }
    }

    fn draw_line(&mut self, x1: u32, y1: u32, x2: u32, y2: u32, color: Color) {
        let x2 = if x1 == x2 { x2 + 1 } else { x2 };
        let y2 = if y1 == y2 { y2 + 1 } else { y2 };

        // bresenham's line drawing
        let dx = (x2 as i32 - x1 as i32).abs();
        let dy = (y2 as i32 - y1 as i32).abs();
        let sx = if x1 < x2 { 1i32 } else { -1 };
        let sy = if y1 < y2 { 1i32 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1 as i32;
        let mut y = y1 as i32;

        while x != x2 as i32 && y != y2 as i32 {
            self.blend_pixel(x as _, y as _, color);

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_text(&mut self, string: &str, x: u32, y: u32, text_alignment: TextAlignment) {
        let font = FONT.as_scaled(FONT.pt_to_px_scale(10.0).unwrap());

        let total_width = string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>();
        let x = match text_alignment {
            TextAlignment::Left => x,
            TextAlignment::Center => x - (total_width / 2.0) as u32,
            TextAlignment::Right => x - total_width as u32,
        };

        let mut position = 0.0;
        for c in string.chars() {
            if c.is_control() {
                continue;
            }

            let glyph = font.scaled_glyph(c);
            let h_advance = font.h_advance(glyph.id);

            if let Some(outlined_glyph) = font.outline_glyph(glyph) {
                outlined_glyph.draw(|glyph_x: u32, glyph_y, c| {
                    self.blend_pixel(
                        x + (glyph_x as f32 + position) as u32,
                        y + glyph_y,
                        Color {
                            a: (c * 255.0) as u8,
                            r: 0,
                            g: 0,
                            b: 0,
                        },
                    )
                });
            }

            position += h_advance;
        }
    }

    fn draw_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for x in x..x + w {
            if x >= self.image_buffer.width() {
                continue;
            }
            self.put_pixel(x, y, color);
            self.put_pixel(x, y + h - 1, color);
        }
        for y in y..y + h {
            if y >= self.image_buffer.height() {
                continue;
            }
            self.put_pixel(x, y, color);
            self.put_pixel(x + w - 1, y, color);
        }
    }

    fn fill_rect(&mut self, x: u32, y: u32, w: u32, h: u32, color: Color) {
        for y in y..y + h {
            for x in x..x + w {
                if x >= self.image_buffer.width() || y >= self.image_buffer.height() {
                    continue;
                }
                self.put_pixel(x, y, color);
            }
        }
    }

    fn put_pixel(&mut self, x: u32, y: u32, color: Color) {
        self.image_buffer.put_pixel(x, y, color)
    }
}

pub struct Clip {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Clip {
    pub fn intersect(&self, other: &Clip) -> Clip {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + self.width).min(other.x + other.width) - x;
        let height = (self.y + self.height).min(other.y + other.height) - y;

        Clip { x, y, width, height }
    }
}

pub fn decode_image(data: &[u8]) -> Result<Box<dyn Image>> {
    use std::io::Cursor;

    let image = ImageReader::new(Cursor::new(&data))
        .with_guessed_format()
        .map_err(|x| WieError::FatalError(x.to_string()))?
        .decode()
        .map_err(|x| WieError::FatalError(x.to_string()))?;
    let rgba = image.into_rgba8();

    let data = rgba.pixels().flat_map(|x| [x.0[2], x.0[1], x.0[0], x.0[3]]).collect::<Vec<_>>();

    Ok(Box::new(VecImageBuffer::<ArgbPixel>::from_raw(
        rgba.width(),
        rgba.height(),
        pod_collect_to_vec(&data),
    )) as Box<_>)
}

pub fn string_width(string: &str, pt_size: f32) -> f32 {
    let font = FONT.as_scaled(FONT.pt_to_px_scale(pt_size).unwrap());

    string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>()
}

#[cfg(test)]
mod tests {
    use wie_util::Result;

    use crate::canvas::{Image, ImageBufferCanvas};

    use super::{ArgbPixel, Canvas, Color, VecImageBuffer};

    #[test]
    fn test_canvas() -> Result<()> {
        let image_buffer = VecImageBuffer::<ArgbPixel>::new(10, 10);
        let mut canvas = ImageBufferCanvas::new(image_buffer);

        canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 });

        let image_buffer = canvas.into_inner();
        let raw = image_buffer.raw();

        assert_eq!(raw.len(), 10 * 10 * 4);
        for i in 0..10 * 10 {
            assert_eq!(raw[i * 4], 0);
            assert_eq!(raw[i * 4 + 1], 0);
            assert_eq!(raw[i * 4 + 2], 0);
            assert_eq!(raw[i * 4 + 3], 255);
        }

        Ok(())
    }
}
