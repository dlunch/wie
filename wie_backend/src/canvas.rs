mod lbmp;

use alloc::{borrow::Cow, boxed::Box, string::ToString, vec, vec::Vec};
use core::mem::size_of;

use ab_glyph::{Font, FontRef, ScaleFont};
use bytemuck::{Pod, cast_slice, pod_collect_to_vec};
use image::ImageReader;
use num_traits::{Num, Zero};

use wie_util::{Result, WieError};

use self::lbmp::decode_lbmp;

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
    fn get_pixel(&self, x: i32, y: i32) -> Color;
    fn raw(&self) -> Cow<'_, [u8]>;
    fn colors(&self) -> Vec<Color>;
}

pub trait ImageBuffer: Send {
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]);
}

#[allow(clippy::too_many_arguments)]
pub trait Canvas: Send {
    fn image(&self) -> &dyn Image;
    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip);
    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color);
    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color);
    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip);
    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip);
    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip);
    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip);
    fn put_pixel(&mut self, x: i32, y: i32, color: Color);
}

pub trait PixelType: Send {
    type DataType: Copy + Pod + Num + Send;
    fn from_color(color: Color) -> Self::DataType;
    fn to_color(raw: Self::DataType) -> Color;
}

pub struct Rgb332Pixel;

impl PixelType for Rgb332Pixel {
    type DataType = u8;

    fn from_color(color: Color) -> Self::DataType {
        let r = (color.r * 7 + 127) / 255;
        let g = (color.g * 7 + 127) / 255;
        let b = (color.b * 3 + 127) / 255;

        (r << 5) | (g << 2) | b
    }

    fn to_color(raw: Self::DataType) -> Color {
        let r = (raw >> 5) & 0x7;
        let g = (raw >> 2) & 0x7;
        let b = raw & 0x3;

        Color {
            a: 0xff,
            r: r * 36,
            g: g * 36,
            b: b * 85,
        }
    }
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
        ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
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
        ((color.a as u32) << 24) | ((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32
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
        ((color.a as u32) << 24) | ((color.b as u32) << 16) | ((color.g as u32) << 8) | color.r as u32
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

    fn get_pixel(&self, x: i32, y: i32) -> Color {
        let raw = self.data[((y as u32) * self.width + (x as u32)) as usize];

        T::to_color(raw)
    }

    fn raw(&self) -> Cow<'_, [u8]> {
        cast_slice(&self.data).into()
    }

    fn colors(&self) -> Vec<Color> {
        self.data.iter().map(|&x| T::to_color(x)).collect()
    }
}

impl<T> ImageBuffer for VecImageBuffer<T>
where
    T: PixelType + 'static,
{
    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
            return;
        }

        let raw = T::from_color(color);

        self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
    }

    fn put_pixels(&mut self, x: i32, y: i32, width: u32, colors: &[Color]) {
        for (i, color) in colors.iter().enumerate() {
            let x = x + (i as i32 % (width as i32));
            let y = y + (i as i32 / (width as i32));

            if x < 0 || y < 0 || (x as u32) >= self.width || (y as u32) >= self.height {
                continue;
            }

            let raw = T::from_color(*color);

            self.data[((y as u32) * self.width + (x as u32)) as usize] = raw;
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

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color) {
        if x < 0 || y < 0 || (x as u32) >= self.image_buffer.width() || (y as u32) >= self.image_buffer.height() {
            return;
        }
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

    fn plot(&mut self, x: i32, y: i32, color: Color, clip: &Clip) {
        if x < 0 || y < 0 || (x as u32) >= self.image_buffer.width() || (y as u32) >= self.image_buffer.height() {
            return;
        }
        if x < clip.x || x >= clip.x + clip.width as i32 || y < clip.y || y >= clip.y + clip.height as i32 {
            return;
        }
        self.put_pixel(x, y, color);
    }

    #[allow(clippy::too_many_arguments)]
    fn stroke_arc(&mut self, cx: f32, cy: f32, a: f32, b: f32, start_deg: f32, sweep_deg: f32, color: Color, clip: &Clip) {
        let radius = a.max(b).max(1.0);
        let sweep_rad = sweep_deg.to_radians();
        let start_rad = start_deg.to_radians();
        let steps = ((sweep_rad.abs() * radius).ceil() as i32 * 2).max(1);

        for i in 0..=steps {
            let theta = start_rad + sweep_rad * (i as f32) / (steps as f32);
            let px = (cx + a * theta.cos()).round() as i32;
            let py = (cy - b * theta.sin()).round() as i32;
            self.plot(px, py, color, clip);
        }
    }
}

/// Whether the point lies within the pie sweep starting at `start_deg` spanning
/// `sweep_deg` degrees (counterclockwise for positive, clockwise for negative).
/// Angles follow the J2ME/WIPI convention: 0° is 3 o'clock, positive is CCW.
fn point_in_sweep(px: f32, py: f32, cx: f32, cy: f32, start_deg: f32, sweep_deg: f32) -> bool {
    if sweep_deg.abs() >= 360.0 {
        return true;
    }
    if sweep_deg == 0.0 {
        return false;
    }

    let dx = px - cx;
    let dy = py - cy;
    if dx * dx + dy * dy < 1.0 {
        return true; // apex of the pie
    }

    // screen y grows downward, so negate dy to match the math convention
    let angle = (-dy).atan2(dx).to_degrees();
    let (start, sweep) = if sweep_deg > 0.0 {
        (start_deg, sweep_deg)
    } else {
        (start_deg + sweep_deg, -sweep_deg)
    };

    (angle - start).rem_euclid(360.0) <= sweep
}

#[allow(clippy::too_many_arguments)]
impl<T> Canvas for ImageBufferCanvas<T>
where
    T: ImageBuffer + Image,
{
    fn image(&self) -> &dyn Image {
        &self.image_buffer
    }

    fn draw(&mut self, dx: i32, dy: i32, w: u32, h: u32, src: &dyn Image, sx: i32, sy: i32, clip: Clip) {
        for y in 0..(h as i32) {
            for x in 0..(w as i32) {
                if sx + x < 0 || sy + y < 0 || sx + x >= src.width() as i32 || sy + y >= src.height() as i32 {
                    continue;
                }
                if dx + x < 0 || dy + y < 0 || dx + x >= self.image_buffer.width() as i32 || dy + y >= self.image_buffer.height() as i32 {
                    continue;
                }
                if dx + x < clip.x || dx + x >= clip.x + (clip.width as i32) || dy + y < clip.y || dy + y >= clip.y + (clip.height as i32) {
                    continue;
                }

                // TODO blend multiple pixels at once for performance
                self.blend_pixel(dx + x, dy + y, src.get_pixel(sx + x, sy + y));
            }
        }
    }

    fn draw_line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32, color: Color) {
        if x1 == x2 && y1 == y2 {
            self.blend_pixel(x1 as _, y1 as _, color);
            return;
        }

        // bresenham's line drawing
        let dx = (x2 - x1).abs();
        let dy = (y2 - y1).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1;
        let mut y = y1;

        while x != x2 || y != y2 {
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

    fn draw_text(&mut self, string: &str, x: i32, y: i32, text_alignment: TextAlignment, color: Color) {
        let size = 10.0; // TODO
        let font = FONT.as_scaled(FONT.pt_to_px_scale(size).unwrap());

        let total_width = string.chars().map(|c| font.h_advance(font.scaled_glyph(c).id)).sum::<f32>();
        let x = match text_alignment {
            TextAlignment::Left => x,
            TextAlignment::Center => x - (total_width / 2.0) as i32,
            TextAlignment::Right => x - total_width as i32,
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
                    let bounds = outlined_glyph.px_bounds();
                    self.blend_pixel(
                        x + (glyph_x as f32 + bounds.min.x + position) as i32,
                        y + (glyph_y as f32 + bounds.min.y + size) as i32,
                        Color {
                            a: (c * 255.0) as u8,
                            r: color.r,
                            g: color.g,
                            b: color.b,
                        },
                    )
                });
            }

            position += h_advance;
        }
    }

    fn draw_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        // TODO use put_pixels
        for x in x..x + (w as i32) {
            if x < 0 || x >= self.image_buffer.width() as i32 {
                continue;
            }
            if x < clip.x || x >= clip.x + clip.width as i32 {
                continue;
            }
            if y < 0 || y >= self.image_buffer.height() as i32 {
                continue;
            }
            if y < clip.y || y >= clip.y + clip.height as i32 {
                continue;
            }

            self.put_pixel(x, y, color);
            self.put_pixel(x, y + (h as i32) - 1, color);
        }
        for y in y..y + (h as i32) {
            if x < 0 || x >= self.image_buffer.width() as i32 {
                continue;
            }
            if x < clip.x || x >= clip.x + clip.width as i32 {
                continue;
            }
            if y < 0 || y >= self.image_buffer.height() as i32 {
                continue;
            }
            if y < clip.y || y >= clip.y + clip.height as i32 {
                continue;
            }

            self.put_pixel(x, y, color);
            self.put_pixel(x + (w as i32) - 1, y, color);
        }
    }

    fn draw_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }

        let a = (w as f32 - 1.0) / 2.0;
        let b = (h as f32 - 1.0) / 2.0;
        let cx = x as f32 + a;
        let cy = y as f32 + b;

        self.stroke_arc(cx, cy, a, b, start_angle as f32, arc_angle as f32, color, &clip);
    }

    fn draw_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }
        if arc_width == 0 || arc_height == 0 {
            return self.draw_rect(x, y, w, h, color, clip);
        }

        let rx = (arc_width.min(w) as f32 / 2.0).max(0.5);
        let ry = (arc_height.min(h) as f32 / 2.0).max(0.5);
        let rxi = rx.round() as i32;
        let ryi = ry.round() as i32;

        let left = x;
        let right = x + w as i32 - 1;
        let top = y;
        let bottom = y + h as i32 - 1;

        for px in (left + rxi)..=(right - rxi) {
            self.plot(px, top, color, &clip);
            self.plot(px, bottom, color, &clip);
        }
        for py in (top + ryi)..=(bottom - ryi) {
            self.plot(left, py, color, &clip);
            self.plot(right, py, color, &clip);
        }

        self.stroke_arc(right as f32 - rx, top as f32 + ry, rx, ry, 0.0, 90.0, color, &clip);
        self.stroke_arc(left as f32 + rx, top as f32 + ry, rx, ry, 90.0, 90.0, color, &clip);
        self.stroke_arc(left as f32 + rx, bottom as f32 - ry, rx, ry, 180.0, 90.0, color, &clip);
        self.stroke_arc(right as f32 - rx, bottom as f32 - ry, rx, ry, 270.0, 90.0, color, &clip);
    }

    fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color, clip: Clip) {
        // TODO use put_pixels
        for y in y..y + (h as i32) {
            for x in x..x + (w as i32) {
                if x >= self.image_buffer.width() as i32 || y >= self.image_buffer.height() as i32 {
                    continue;
                }
                if x < clip.x || x >= clip.x + clip.width as i32 || y < clip.y || y >= clip.y + clip.height as i32 {
                    continue;
                }
                self.put_pixel(x, y, color);
            }
        }
    }

    fn fill_arc(&mut self, x: i32, y: i32, w: u32, h: u32, start_angle: i32, arc_angle: i32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }

        let a = (w as f32 - 1.0) / 2.0;
        let b = (h as f32 - 1.0) / 2.0;
        let cx = x as f32 + a;
        let cy = y as f32 + b;
        let da = a.max(0.5);
        let db = b.max(0.5);
        let start = start_angle as f32;
        let sweep = arc_angle as f32;

        for py in y..y + h as i32 {
            for px in x..x + w as i32 {
                let nx = (px as f32 - cx) / da;
                let ny = (py as f32 - cy) / db;
                if nx * nx + ny * ny > 1.0 {
                    continue;
                }
                if !point_in_sweep(px as f32, py as f32, cx, cy, start, sweep) {
                    continue;
                }
                self.plot(px, py, color, &clip);
            }
        }
    }

    fn fill_round_rect(&mut self, x: i32, y: i32, w: u32, h: u32, arc_width: u32, arc_height: u32, color: Color, clip: Clip) {
        if w == 0 || h == 0 {
            return;
        }
        if arc_width == 0 || arc_height == 0 {
            return self.fill_rect(x, y, w, h, color, clip);
        }

        let rx = (arc_width.min(w) as f32 / 2.0).max(0.5);
        let ry = (arc_height.min(h) as f32 / 2.0).max(0.5);

        let left_center = x as f32 + rx;
        let right_center = (x + w as i32 - 1) as f32 - rx;
        let top_center = y as f32 + ry;
        let bottom_center = (y + h as i32 - 1) as f32 - ry;

        for py in y..y + h as i32 {
            let cy = if (py as f32) < top_center {
                top_center
            } else if (py as f32) > bottom_center {
                bottom_center
            } else {
                py as f32
            };
            for px in x..x + w as i32 {
                let cx = if (px as f32) < left_center {
                    left_center
                } else if (px as f32) > right_center {
                    right_center
                } else {
                    px as f32
                };

                let nx = (px as f32 - cx) / rx;
                let ny = (py as f32 - cy) / ry;
                if nx * nx + ny * ny <= 1.0 {
                    self.plot(px, py, color, &clip);
                }
            }
        }
    }

    fn put_pixel(&mut self, x: i32, y: i32, color: Color) {
        self.image_buffer.put_pixel(x, y, color)
    }
}

pub struct Clip {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl Clip {
    pub fn intersect(&self, other: &Clip) -> Clip {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let width = (self.x + (self.width as i32)).min(other.x + (other.width as i32)) - x;
        let height = (self.y + (self.height as i32)).min(other.y + (other.height as i32)) - y;

        Clip {
            x,
            y,
            width: width as _,
            height: height as _,
        }
    }
}

pub fn decode_image(data: &[u8]) -> Result<Box<dyn Image>> {
    extern crate std; // XXX

    use std::io::Cursor;

    if data[0] == b'L' && data[1] == b'B' && data[2] == b'M' && data[3] == b'P' {
        return decode_lbmp(data);
    }

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

    use crate::canvas::{Clip, Image, ImageBufferCanvas};

    use super::{ArgbPixel, Canvas, Color, VecImageBuffer};

    #[test]
    fn test_canvas() -> Result<()> {
        let image_buffer = VecImageBuffer::<ArgbPixel>::new(10, 10);
        let mut canvas = ImageBufferCanvas::new(image_buffer);

        let clip = Clip {
            x: 0,
            y: 0,
            width: 10,
            height: 10,
        };
        canvas.fill_rect(0, 0, 10, 10, Color { r: 0, g: 0, b: 0, a: 255 }, clip);

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

    fn full_clip(size: u32) -> Clip {
        Clip {
            x: 0,
            y: 0,
            width: size,
            height: size,
        }
    }

    const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    fn is_set(image: &impl Image, x: i32, y: i32) -> bool {
        let c = image.get_pixel(x, y);
        c.r != 0 || c.g != 0 || c.b != 0
    }

    #[test]
    fn test_fill_arc_full_circle() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.fill_arc(0, 0, 32, 32, 0, 360, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 16), "center should be filled");
        assert!(!is_set(&image, 0, 0), "corner should be outside the ellipse");
        assert!(!is_set(&image, 31, 31), "corner should be outside the ellipse");
    }

    #[test]
    fn test_fill_arc_quadrant() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        // 0..90 deg sweeps from 3 o'clock CCW to 12 o'clock => upper-right quadrant
        canvas.fill_arc(0, 0, 32, 32, 0, 90, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 24, 8), "upper-right quadrant should be filled");
        assert!(!is_set(&image, 8, 8), "upper-left quadrant should be empty");
        assert!(!is_set(&image, 8, 24), "lower-left quadrant should be empty");
        assert!(!is_set(&image, 24, 24), "lower-right quadrant should be empty");
    }

    #[test]
    fn test_fill_arc_negative_sweep_is_clockwise() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        // 0..-90 deg sweeps CW from 3 o'clock to 6 o'clock => lower-right quadrant
        canvas.fill_arc(0, 0, 32, 32, 0, -90, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 24, 24), "lower-right quadrant should be filled");
        assert!(!is_set(&image, 24, 8), "upper-right quadrant should be empty");
    }

    #[test]
    fn test_draw_arc_endpoint() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.draw_arc(0, 0, 32, 32, 0, 360, WHITE, full_clip(32));
        let image = canvas.into_inner();

        // bounding box 32x32 => center (15.5, 15.5), radius 15.5; nominal endpoints round to:
        assert!(is_set(&image, 31, 16), "3 o'clock (0 deg) point");
        assert!(is_set(&image, 16, 0), "12 o'clock (90 deg) point");
        assert!(is_set(&image, 0, 16), "9 o'clock (180 deg) point");
        assert!(is_set(&image, 16, 31), "6 o'clock (270 deg) point");
        assert!(!is_set(&image, 16, 16), "arc outline must not fill the interior");
    }

    #[test]
    fn test_draw_round_rect_outline() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.draw_round_rect(0, 0, 32, 32, 16, 16, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 0), "top straight edge");
        assert!(is_set(&image, 16, 31), "bottom straight edge");
        assert!(is_set(&image, 0, 16), "left straight edge");
        assert!(is_set(&image, 31, 16), "right straight edge");
        assert!(!is_set(&image, 0, 0), "rounded corner cut away");
        assert!(!is_set(&image, 16, 16), "outline must not fill the interior");
    }

    #[test]
    fn test_fill_arc_degenerate_width() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(8, 8));
        canvas.fill_arc(3, 0, 1, 8, 0, 360, WHITE, full_clip(8));
        let image = canvas.into_inner();

        for y in 0..8 {
            assert!(is_set(&image, 3, y), "degenerate 1px-wide column should be fully filled (y={y})");
        }
    }

    #[test]
    fn test_fill_round_rect_corners_empty() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        canvas.fill_round_rect(0, 0, 32, 32, 16, 16, WHITE, full_clip(32));
        let image = canvas.into_inner();

        assert!(is_set(&image, 16, 16), "center should be filled");
        assert!(is_set(&image, 16, 0), "straight top edge should be filled");
        assert!(!is_set(&image, 0, 0), "rounded corner should be empty");
        assert!(!is_set(&image, 31, 0), "rounded corner should be empty");
    }

    #[test]
    fn test_arc_respects_clip() {
        let mut canvas = ImageBufferCanvas::new(VecImageBuffer::<ArgbPixel>::new(32, 32));
        let clip = Clip {
            x: 0,
            y: 0,
            width: 16,
            height: 32,
        };
        canvas.fill_arc(0, 0, 32, 32, 0, 360, WHITE, clip);
        let image = canvas.into_inner();

        assert!(is_set(&image, 8, 16), "inside clip should be filled");
        assert!(!is_set(&image, 24, 16), "outside clip must not be filled");
    }
}
